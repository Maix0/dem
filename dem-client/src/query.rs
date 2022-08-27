use bounce::{prelude::*, query::*};
use std::ops::Deref;

#[macro_export]
macro_rules! run {
    ($($i:ident,)* {$e:expr}) => {
        wasm_bindgen_futures::spawn_local({
            $(let $i = $i.clone();)*
            async move {
                let _ = $e.await;
            }
        })
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuildEmoteQuery(Vec<dem_http::models::EmojiItem>);

impl Deref for GuildEmoteQuery {
    type Target = Vec<dem_http::models::EmojiItem>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait::async_trait(?Send)]
impl Query for GuildEmoteQuery {
    type Input = u64;
    type Error = crate::CloneError<dem_http::apis::default_api::ApiGetGuildEmojisError>;

    async fn query(states: &BounceStates, input: std::rc::Rc<Self::Input>) -> QueryResult<Self> {
        dem_http::apis::default_api::api_get_guild_emojis(
            &*states.get_atom_value::<crate::APIConfig>(),
            *input,
        )
        .await
        .map(|v| Self(v.ok).into())
        .map_err(Into::into)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurrentUserQuery(Option<dem_http::models::OkResponseForNullableUserLoginOk>);

impl Deref for CurrentUserQuery {
    type Target = Option<dem_http::models::OkResponseForNullableUserLoginOk>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait::async_trait(?Send)]
impl Query for CurrentUserQuery {
    type Input = ();
    type Error = crate::CloneError<dem_http::apis::default_api::ApiGetCurrentUserError>;

    async fn query(states: &BounceStates, _input: std::rc::Rc<Self::Input>) -> QueryResult<Self> {
        dem_http::apis::default_api::api_get_current_user(
            &*states.get_atom_value::<crate::APIConfig>(),
        )
        .await
        .map(|v| Self(v.ok.map(|v| *v)).into())
        .map_err(Into::into)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserGuildsQuery(Vec<dem_http::models::PartialGuildWithPermission>);

impl Deref for UserGuildsQuery {
    type Target = Vec<dem_http::models::PartialGuildWithPermission>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait::async_trait(?Send)]
impl Query for UserGuildsQuery {
    type Input = ();
    type Error = crate::CloneError<dem_http::apis::default_api::ApiGetOverlappingGuildsError>;

    async fn query(states: &BounceStates, _input: std::rc::Rc<Self::Input>) -> QueryResult<Self> {
        dem_http::apis::default_api::api_get_overlapping_guilds(
            &*states.get_atom_value::<crate::APIConfig>(),
        )
        .await
        .map(|v| Self(v.ok).into())
        .map_err(Into::into)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UploadEmojiToStoreMutation(String);

impl Deref for UploadEmojiToStoreMutation {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UploadEmojiToStoreMutationParams {
    pub guild_id: u64,
    pub emoji_name: String,
    pub emoji_data: Vec<u8>,
    pub content_type: &'static str,
}

#[async_trait::async_trait(?Send)]
impl Mutation for UploadEmojiToStoreMutation {
    type Input = UploadEmojiToStoreMutationParams;
    type Error = crate::CloneError<dem_http::apis::default_api::ImageUploadEmojiToStoreError>;

    async fn run(states: &BounceStates, input: std::rc::Rc<Self::Input>) -> MutationResult<Self> {
        let UploadEmojiToStoreMutationParams {
            ref guild_id,
            ref emoji_name,
            ref emoji_data,
            content_type,
        } = &*input;
        dem_http::apis::default_api::image_upload_emoji_to_store(
            &*states.get_atom_value::<crate::APIConfig>(),
            *guild_id,
            emoji_name.as_str(),
            content_type,
            emoji_data.clone(),
        )
        .await
        .map(|v| Self(v.ok).into())
        .map_err(Into::into)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GetUploadedEmojisQuery(Vec<dem_http::models::ImageDataApi>);

impl Deref for GetUploadedEmojisQuery {
    type Target = Vec<dem_http::models::ImageDataApi>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait::async_trait(?Send)]
impl Query for GetUploadedEmojisQuery {
    type Input = u64;
    type Error = crate::error::CloneError<dem_http::apis::default_api::ImageImageListError>;

    async fn query(states: &BounceStates, input: std::rc::Rc<Self::Input>) -> QueryResult<Self> {
        dem_http::apis::default_api::image_image_list(
            &*states.get_atom_value::<crate::APIConfig>(),
            *input,
        )
        .await
        .map(|v| Self(v.ok).into())
        .map_err(Into::into)
    }
}
