use schemars::JsonSchema;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, JsonSchema)]
pub struct UserLogin {
    pub id: u64,
    pub avatar: Option<String>,
    pub username: String,
    pub discriminator: String,
}


