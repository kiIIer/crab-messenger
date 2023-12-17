use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AccessToken {
    #[serde(rename = "sub")]
    pub id: String,
}
