use rocket_okapi::JsonSchema;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct EmojiItem {
    #[serde(deserialize_with = "deserialize_str")]
    pub id: u64,
    pub animated: bool,
    pub available: bool,
    pub managed: bool,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct StickerItem {
    #[serde(deserialize_with = "deserialize_str")]
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct PartialGuild {
    #[serde(deserialize_with = "deserialize_str")]
    pub id: u64,
    pub name: String,
    pub icon: String,
    pub emojis: Vec<EmojiItem>,
    pub stickers: Vec<StickerItem>,
    pub description: Option<String>,
    pub members: Vec<GuildMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GuildMember {
    pub user: User,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_str")]
    pub permissions: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct User {
    #[serde(deserialize_with = "deserialize_str")]
    pub id: u64, 
}


#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct PartialGuildWithPermission {
    #[serde(deserialize_with = "deserialize_str")]
    pub id: u64,
    pub name: String,
    pub icon: String,
    pub emojis: Vec<EmojiItem>,
    pub stickers: Vec<StickerItem>,
    pub description: Option<String>,
    pub permissions: u64,
}

impl PartialGuildWithPermission {
    pub fn from_partial_guild(guild: PartialGuild, permissions: u64) -> Self {
        PartialGuildWithPermission {
            permissions,
            id: guild.id,
            name: guild.name,
            icon: guild.icon,
            emojis: guild.emojis,
            stickers: guild.stickers,
            description: guild.description,
        }
    }
}

fn deserialize_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = <String>::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}
