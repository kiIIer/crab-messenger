use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::PgConnection;
use dotenv::dotenv;
use shaku::{module, Component, Interface};
use std::env;
use std::sync::Arc;

pub trait DBConnectionManager: Interface {
    fn get_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, r2d2::Error>;
}

#[derive(Component)]
#[shaku(interface=DBConnectionManager)]
pub struct DBConnectionManagerImpl {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DBConnectionManager for DBConnectionManagerImpl {
    fn get_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, r2d2::Error> {
        self.pool.get()
    }
}

module! {
    pub DBConnectionManagerModule {
        components = [DBConnectionManagerImpl],
        providers = [],
    }
}

pub fn build_db_connection_manager_module() -> Arc<DBConnectionManagerModule> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder().build(manager).unwrap();
    Arc::new(
        DBConnectionManagerModule::builder()
            .with_component_parameters::<DBConnectionManagerImpl>(
                DBConnectionManagerImplParameters { pool },
            )
            .build(),
    )
}
