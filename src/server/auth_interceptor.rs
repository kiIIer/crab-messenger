use std::str::FromStr;
use std::sync::Arc;

use diesel::prelude::*;
use diesel::{PgConnection, RunQueryDsl};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shaku::{module, Component, Interface};
use tonic::metadata::{MetadataMap, MetadataValue};
use tonic::{Request, Status};
use tracing::field::debug;
use tracing::{debug, error, info, warn};

use crate::server::crab_messenger::user_manager::{
    build_user_manager_module, UserManager, UserManagerModule,
};
use crate::utils::auth::token::AccessToken;
use crate::utils::db_connection_manager::{
    build_db_connection_manager_module, DBConnectionManager, DBConnectionManagerModule,
};
use crate::utils::persistence::schema::users;
use crate::utils::persistence::user::User;

pub trait AuthInterceptorFactory: Interface {
    fn create(&self) -> AuthInterceptor;
}

#[derive(Component)]
#[shaku(interface = AuthInterceptorFactory)]
pub struct AuthInterceptorFactoryImpl {
    #[shaku(inject)]
    db_connection_manager: Arc<dyn DBConnectionManager>,

    #[shaku(inject)]
    user_manager: Arc<dyn UserManager>,

    client_id: String,
    client_secret: String,
    audience: String,
    server_n: String,
    server_e: String,
}

impl AuthInterceptorFactory for AuthInterceptorFactoryImpl {
    fn create(&self) -> AuthInterceptor {
        AuthInterceptor::new(
            self.db_connection_manager.clone(),
            self.user_manager.clone(),
            self.client_id.clone(),
            self.client_secret.clone(),
            self.audience.clone(),
            self.server_n.clone(),
            self.server_e.clone(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct UserInfo {
    email: String,
}

#[derive(Clone)]
pub struct AuthInterceptor {
    db_connection_manager: Arc<dyn DBConnectionManager>,

    user_manager: Arc<dyn UserManager>,

    client_id: String,
    client_secret: String,
    audience: String,
    server_n: String,
    server_e: String,
}

#[derive(Deserialize)]
struct Auth0TokenResponse {
    access_token: String,
}

impl AuthInterceptor {
    pub fn new(
        db_connection_manager: Arc<dyn DBConnectionManager>,
        user_manager: Arc<dyn UserManager>,
        client_id: String,
        client_secret: String,
        audience: String,
        server_n: String,
        server_e: String,
    ) -> Self {
        Self {
            db_connection_manager,
            user_manager,
            client_id,
            client_secret,
            audience,
            server_n,
            server_e,
        }
    }

    async fn get_auth0_access_token(&self) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let token_url = "https://crab-messenger.eu.auth0.com/oauth/token";
        let request_body = json!({
            "client_id": self.client_id,
            "client_secret": self.client_secret,
            "audience": self.audience,
            "grant_type": "client_credentials"
        });

        let response = client
            .post(token_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|err| {
                error!("Failed to retrieve Auth0 access token: {}", err);
                err
            })?;

        let token_response: Auth0TokenResponse = response.json().await.map_err(|err| {
            error!("Failed to parse Auth0 token response: {}", err);
            anyhow::Error::new(err)
        })?;

        info!(
            "Retrieved Auth0 access token {}",
            token_response.access_token
        );
        Ok(token_response.access_token)
    }

    #[tracing::instrument(skip(self))]
    async fn get_user_info(&self, user_id: &str) -> anyhow::Result<String> {
        info!("Getting user info");
        let access_token = self.get_auth0_access_token().await?;

        let client = reqwest::Client::new();
        let url = format!("{}users", self.audience);

        let from_params = [
            ("q", format!("{}", user_id)),
            ("search_engine", "v3".to_string()),
        ];

        let response = client
            .get(url)
            .query(&from_params)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|err| {
                error!("Failed to get user info: {}", err);
                err
            })?;
        debug!("Response: {:?}", response);
        let response_body = response.text().await.map_err(|err| {
            error!("Failed to get user info: {}", err);
            err
        })?;

        debug!("Response body: {}", response_body);

        let users: Vec<UserInfo> = serde_json::from_str(&response_body).map_err(|err| {
            error!("Failed to parse user info: {}", err);
            anyhow::Error::new(err)
        })?;

        users.get(0).map(|user| user.email.clone()).ok_or_else(|| {
            anyhow::Error::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "User not found",
            ))
        })
    }

    async fn find_by_id(
        &self,
        conn: &mut PgConnection,
        user_id: &str,
    ) -> anyhow::Result<Option<User>> {
        users::table
            .filter(users::id.eq(user_id))
            .first::<User>(conn)
            .optional()
            .map_err(|err| {
                error!("Failed to query user: {}", err);
                anyhow::Error::new(err)
            })
    }

    #[tracing::instrument(skip(self))]
    async fn check_user(&self, user_id: &str) -> anyhow::Result<()> {
        let mut conn = self.db_connection_manager.get_connection()?;

        if let Some(user) = self.find_by_id(&mut conn, user_id).await? {
            Ok(())
        } else {
            let email = self.get_user_info(user_id).await?;

            debug!("Creating user with email: {}", &email);
            self.user_manager
                .create_user(User {
                    id: user_id.to_string(),
                    email,
                })
                .await?;

            Ok(())
        }
    }

    #[tracing::instrument(skip(self, req))]
    pub async fn intercept(&self, mut req: Request<()>) -> Result<Request<()>, Status> {
        info!("Intercepting request");
        let self_clone = self.clone();

        let headers = req.metadata();
        let token = headers
            .get("authorization")
            .to_owned()
            .and_then(|val| val.to_str().ok())
            .ok_or_else(|| {
                warn!("No authorization token found");
                Status::unauthenticated("No authorization token found")
            })?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&["crab-api", "https://crab-messenger.eu.auth0.com/userinfo"]);

        let token_message = decode::<AccessToken>(
            &token,
            &DecodingKey::from_rsa_components(&self.server_n, &self.server_e)
                .expect("Failed to decode public key"),
            &validation,
        )
        .map_err(|e| {
            error!("Failed to decode token: {}", e);
            Status::unauthenticated("Failed to decode token")
        })?;

        let access_token = token_message.claims;
        let user_id = &access_token.id;

        match self_clone.check_user(user_id).await {
            Ok(_) => {
                info!("User verified or created successfully");
            }
            Err(e) => {
                error!("Failed to verify or create user: {}", e);
                return Err(Status::internal("Failed to verify or create user"));
            }
        }

        let mut metadata_map = MetadataMap::new();
        let user_id_meta = MetadataValue::from_str(user_id).map_err(|_| {
            error!("Invalid user_id");
            Status::invalid_argument("Invalid user_id")
        })?;

        metadata_map.insert("user_id", user_id_meta);
        *req.metadata_mut() = metadata_map;

        Ok(req)
    }
}

module! {
    pub AuthInterceptorModule {
        components = [AuthInterceptorFactoryImpl],
        providers = [],
        use DBConnectionManagerModule{
            components = [dyn DBConnectionManager],
            providers = []
        },
        use UserManagerModule{
            components = [dyn UserManager],
            providers = []
        }
    }
}

pub fn build_auth_interceptor_module() -> Arc<AuthInterceptorModule> {
    dotenv::dotenv()
        .map_err(|err| {
            error!("Failed to load .env file: {}", err);
            panic!("Failed to load .env file");
        })
        .ok();

    Arc::new(
        AuthInterceptorModule::builder(
            build_db_connection_manager_module(),
            build_user_manager_module(),
        )
        .with_component_parameters::<AuthInterceptorFactoryImpl>(
            AuthInterceptorFactoryImplParameters {
                client_id: dotenv::var("AUTH0_CLIENT_ID")
                    .map_err(|err| {
                        error!("AUTH0_CLIENT_ID must be set in .env file: {}", err);
                        panic!("AUTH0_CLIENT_ID must be set in .env file");
                    })
                    .unwrap(),
                client_secret: dotenv::var("AUTH0_CLIENT_SECRET")
                    .map_err(|err| {
                        error!("AUTH0_CLIENT_SECRET must be set in .env file: {}", err);
                        panic!("AUTH0_CLIENT_SECRET must be set in .env file");
                    })
                    .unwrap(),
                audience: dotenv::var("AUTH0_AUDIENCE")
                    .map_err(|err| {
                        error!("AUTH0_AUDIENCE must be set in .env file: {}", err);
                        panic!("AUTH0_AUDIENCE must be set in .env file");
                    })
                    .unwrap(),
                server_n: dotenv::var("AUTH0_SERVER_N")
                    .map_err(|err| {
                        error!("AUTH0_SERVER_N must be set in .env file: {}", err);
                        panic!("AUTH0_SERVER_N must be set in .env file");
                    })
                    .unwrap(),
                server_e: dotenv::var("AUTH0_SERVER_E")
                    .map_err(|err| {
                        error!("AUTH0_SERVER_E must be set in .env file: {}", err);
                        panic!("AUTH0_SERVER_E must be set in .env file");
                    })
                    .unwrap(),
            },
        )
        .build(),
    )
}

#[cfg(test)]
mod test {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    use crate::utils::auth::token::AccessToken;

    #[test]
    fn test_key() {
        let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJnb29nbGUtb2F1dGgyfDEwODcwNjE4MTUyMTYyMjc4MzgzMyIsImF1ZCI6WyJjcmFiLWFwaSIsImh0dHBzOi8vY3JhYi1tZXNzZW5nZXIuZXUuYXV0aDAuY29tL3VzZXJpbmZvIl0sImlhdCI6MTcwMjQ4Mzc5MCwiZXhwIjoxNzAyNTcwMTkwLCJhenAiOiJhODVYYzVKeXFOM2c1N1dQVUxWdTRqVE92amhOV2JXbSIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgb2ZmbGluZV9hY2Nlc3MifQ.aj18vt4rZ-r_JRkekzOqIdGP4ekerDLJax72FLiBo3k00NhE1vgdJMLWZNye-GXNsOIVBM0dtnjwdt2xKCGTrzZM1QKa7VHoSG3ozwOdVYry685qmnn3oQ-8I94ew-syk9s2M1Kh3w1gX0XF-ubU2WC7jiNTDnubrlK94D3ynzX163-TSZ5k8mTcSrKFSf0IvvmT7cQ0PM8cMfWOs61WSIjA5x28ExpTTY8nBiw3ZnsvCWEkozzE4oQI59dEgn1STerBh1gEKTlPzXIwezTKMI6ulUtvMZE-ogOokwkKo5rK91D_A4Av_68JJeMI9eBsxcmo9t656N1p_hq3bQXqSg";
        let n = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJnb29nbGUtb2F1dGgyfDEwODcwNjE4MTUyMTYyMjc4MzgzMyIsImF1ZCI6WyJjcmFiLWFwaSIsImh0dHBzOi8vY3JhYi1tZXNzZW5nZXIuZXUuYXV0aDAuY29tL3VzZXJpbmZvIl0sImlhdCI6MTcwMjQ4Mzc5MCwiZXhwIjoxNzAyNTcwMTkwLCJhenAiOiJhODVYYzVKeXFOM2c1N1dQVUxWdTRqVE92amhOV2JXbSIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgb2ZmbGluZV9hY2Nlc3MifQ.aj18vt4rZ-r_JRkekzOqIdGP4ekerDLJax72FLiBo3k00NhE1vgdJMLWZNye-GXNsOIVBM0dtnjwdt2xKCGTrzZM1QKa7VHoSG3ozwOdVYry685qmnn3oQ-8I94ew-syk9s2M1Kh3w1gX0XF-ubU2WC7jiNTDnubrlK94D3ynzX163-TSZ5k8mTcSrKFSf0IvvmT7cQ0PM8cMfWOs61WSIjA5x28ExpTTY8nBiw3ZnsvCWEkozzE4oQI59dEgn1STerBh1gEKTlPzXIwezTKMI6ulUtvMZE-ogOokwkKo5rK91D_A4Av_68JJeMI9eBsxcmo9t656N1p_hq3bQXqSg";
        let e = "AQAB";

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&["crab-api", "https://crab-messenger.eu.auth0.com/userinfo"]);

        let message = decode::<AccessToken>(
            &token,
            &DecodingKey::from_rsa_components(n, e).expect("Failed to decode public key"),
            &validation,
        );

        println!("{:?}", message);
        assert!(message.is_ok());
    }
}
