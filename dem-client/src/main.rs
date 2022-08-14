#[macro_use]
extern crate weblog;
extern crate dem_http;
extern crate gloo_utils;
extern crate material_yew;
extern crate stylist;
extern crate yew;
extern crate yew_hooks;

use std::ops::Deref;

use material_yew::{top_app_bar_fixed::*, *};
use stylist::yew::*;
use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_router::prelude::*;

mod logged;
mod style;
mod theme;

#[derive(Clone, Debug)]
enum CloneError<T> {
    ResponseError(dem_http::apis::ResponseContent<T>),
    Io(String),
    Reqwest(String),
    Serde(String),
}

impl<T: Clone> std::convert::From<dem_http::apis::Error<T>> for CloneError<T> {
    fn from(c: dem_http::apis::Error<T>) -> Self {
        match c {
            dem_http::apis::Error::Io(e) => Self::Io(e.to_string()),
            dem_http::apis::Error::Serde(e) => Self::Serde(e.to_string()),
            dem_http::apis::Error::Reqwest(e) => Self::Reqwest(e.to_string()),
            dem_http::apis::Error::ResponseError(r) => Self::ResponseError(r),
        }
    }
}
impl<T: Clone> CloneError<T> {
    fn from_error(err: dem_http::apis::Error<T>) -> Self {
        err.into()
    }
}

impl<T> CloneError<T> {}

macro_rules! gen_error_name {
    ($($err:tt),*) => {
        $(
        #[allow(dead_code)]
        impl CloneError<$err> {
            fn catergory(&self) -> &'static str {
                match self {
                    CloneError::Io(_) => "Io",
                    CloneError::Serde(_) => "Api Deserialization",
                    CloneError::Reqwest(_) => "Http",
                    CloneError::ResponseError(r) => {
                        if let Some($err::Status400(err_response)) = &r.entity {
                            match err_response.err.code {
                                dem_http::models::error::Error::Internal => {
                                    "Internal"
                                }
                                dem_http::models::error::Error::DiscordAPI => {
                                    "Discord API"
                                }
                                dem_http::models::error::Error::Unauthorized => {
                                    "Unauthorize"
                                }
                            }
                        } else {
                            "Unknown"
                        }
                    }
                }
            }
            fn detail(&self) -> String {
                match self {
                    CloneError::Io(e) => e.to_string(),
                    CloneError::Serde(e) => e.to_string(),
                    CloneError::Reqwest(e) => e.to_string(),
                    CloneError::ResponseError(ref r) => {
                        if let Some($err::Status400(err_response)) = &r.entity {
                            err_response.err.description.clone()
                        } else {
                            "No description".to_string()
                        }

                    }
                }
            }
        }
        )*
    };
}
use dem_http::apis::default_api::*;
gen_error_name!(
    ApiGetCurrentUserError,
    ApiGetGuildEmojisError,
    ApiGetGuildStickersError,
    ApiGetOverlappingGuildsError
);

#[derive(Debug, Clone, Default)]
pub struct APIConfig(dem_http::apis::configuration::Configuration);

impl PartialEq<APIConfig> for APIConfig {
    fn eq(&self, _: &APIConfig) -> bool {
        true
    }
}

impl Deref for APIConfig {
    type Target = dem_http::apis::configuration::Configuration;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

type AppLink = Link<Routes>;

fn enable_auto() -> UseAsyncOptions {
    UseAsyncOptions::enable_auto()
}

#[function_component(App)]
fn app() -> Html {
    let config = use_state(APIConfig::default);
    let drawer = use_state(|| false);
    let user_login = use_async_with_options(
        {
            let config = config.clone();
            async move {
                dem_http::apis::default_api::api_get_current_user(&config)
                    .await
                    .map_err(CloneError::from_error)
            }
        },
        enable_auto(),
    );
    let guilds = use_async_with_options(
        {
            let config = config.clone();
            async move {
                dem_http::apis::default_api::api_get_overlapping_guilds(&config)
                    .await
                    .map_err(CloneError::from_error)
            }
        },
        enable_auto(),
    );

    let toggle_drawer = {
        let drawer = drawer.clone();
        Callback::from(move |()| {
            drawer.set(!*drawer);
        })
    };

    let onclick = {
        let drawer = drawer.clone();
        Callback::from(move |()| {
            drawer.set(false);
        })
    };

    html! {
        <>
        <style::MainStyle />
        <theme::MatThemeSetter ..theme::MatThemeSetterProps::DARK_THEME/>
        <ContextProvider<APIConfig> context={(*config).clone()}>
            <HashRouter>
                <MatDrawer open={*drawer} drawer_type="dismissible">
                    <div onclick={onclick.clone().reform(|_| ())}>
                        <MatIconButton icon="close" label="Close" />
                    </div>

                    {
                        if let Some(guilds) = &guilds.data {
                            guilds.ok.iter().map(|g|
                                html!{<GuildListItem guild={g.clone()} onclick={onclick.clone()}/>}
                            ).collect::<Html>()
                        } else if let Some(error) = &guilds.error {
                            html!{<ErrorComponent name={error.catergory()} description={error.detail()} />}
                        } else {
                            html!{}
                        }
                    }
                </MatDrawer>
                <MatTopAppBarFixed onnavigationiconclick={toggle_drawer}>
                    <MatTopAppBarNavigationIcon>
                        <MatIconButton icon="menu"></MatIconButton>
                    </MatTopAppBarNavigationIcon>

                    <MatTopAppBarTitle>
                        {"Discord Emojis Manager"}
                    </MatTopAppBarTitle>
                    <MatTopAppBarActionItems>
                        {
                            if let Some(dem_http::models::OkResponseForNullableUserLogin {ok: Some(user)}) = &user_login.data {
                                html! {
                                    <div> {format!("Logged in as {}", user.username)} </div>
                                }
                            } else if let Some(error) = &user_login.error {
                                html!{<ErrorComponent name={error.catergory()} description={error.detail()} />}
                            } else {
                                html! {
                                    <a href="/api/auth">
                                        <MatButton label="Login" icon={yew::virtual_dom::AttrValue::from("login")} unelevated=true trailing_icon=true/>
                                    </a>
                                }
                            }
                        }
                    </MatTopAppBarActionItems>

                </MatTopAppBarFixed>
                {
                    if let Some(guilds) = guilds.data.clone() {
                        html! {<Switch<Routes> render={Callback::<Routes, Html>::from(move |r| switch(&guilds, r))} />}
                    } else {
                        html!{"Loading..."}
                    }
                }
            </HashRouter>
        </ContextProvider<APIConfig>>
    </>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct ErrorComponentProps {
    name: String,
    description: String,
}

#[function_component(ErrorComponent)]
fn error_component(ErrorComponentProps { name, description }: &ErrorComponentProps) -> Html {
    html! {
        <MatSnackbar label_text={format!("{} Error: {}", name, description)}>
            <span slot="action">
                <MatButton label="RETRY" />
            </span>

            <span class="snackbar-dismiss-slot" slot="dismiss">
                <MatIconButton icon="close" />
            </span>
        </MatSnackbar>
    }
}

#[derive(Debug, Clone, Properties, PartialEq)]
struct Guild {
    guild: dem_http::models::PartialGuildWithPermission,
    onclick: Callback<()>,
}

#[styled_component(GuildListItem)]
fn guild_list_item(
    Guild {
        guild: dem_http::models::PartialGuildWithPermission { id, name, icon, .. },
        onclick,
    }: &Guild,
) -> Html {
    html! {
        <AppLink to={Routes::Guild {id: *id} }>
        <div class={css!("padding-top: 0px; margin-top: 0px; display: flex; flex-direction: row; flex-wrap: nowrap; justify-content: flex-start; align-items: center; height: 4em;")}
            onclick={onclick.reform(|_| ())}>
            <img class={css!("width: 3em; height: 3em; border-radius: 50%; padding-right: 1em;")}
                src={yew::virtual_dom::AttrValue::from(format!("https://cdn.discordapp.com/icons/{id}/{icon}.png?size=1024"))} />
            <span class={css!("text-decoration: none; color: var(--mdc-theme-on-surface);")}>{name}</span>
        </div>
        </AppLink>
    }
}

#[derive(Debug, Clone, PartialEq, Routable)]
enum Routes {
    #[at("/")]
    Main,
    #[at("/guild/:id")]
    Guild { id: u64 },
}

fn switch(
    guilds: &dem_http::models::OkResponseForArrayOfPartialGuildWithPermission,
    r: Routes,
) -> Html {
    match r {
        Routes::Main => html! {
            {"Main App"}
        },
        Routes::Guild { id } => {
            if guilds.ok.iter().any(|g| g.id == id) {
                html! {
                    <GuildEmojiList {id} />
                }
            } else {
                html! {
                    {"No guild found"}
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct EmojiListProps {
    id: u64,
}

#[styled_component(GuildEmojiList)]
fn emoji_list(props: &EmojiListProps) -> Html {
    let api_config = use_context::<APIConfig>().unwrap();
    let emojis = use_async_with_options(
        {
            let id = props.id;
            async move {
                dem_http::apis::default_api::api_get_guild_emojis(&api_config, id)
                    .await
                    .map_err(CloneError::from_error)
            }
        },
        enable_auto(),
    );
    html! {
        {
            if emojis.loading {
                html!{"Loading"}
            }
            else if let Some(emojis) = &emojis.data {
                html! {
                    <div class={css!("display: flex; flex-direction: row; flex-wrap: wrap; justify-content: space-between;")}>
                    {
                        emojis.ok.iter().map(|e| html! {
                            <EmojiListItem inner={e.clone()}/>
                        }).collect::<Html>()
                    }
                    </div>
                }
            } else if let Some(error) = &emojis.error {
                html!{<ErrorComponent name={error.catergory()} description={error.detail()} />}
            } else {
                html! {<ErrorComponent name={"DEV"} description={"Maix fucked up"}/> }
            }
        }
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
struct EmojiListItemProps {
    pub inner: dem_http::models::EmojiItem,
}

#[styled_component(EmojiListItem)]
fn emoji_list_item(props: &EmojiListItemProps) -> Html {
    html! {
        <div class={css!("display: flex; flex-direction: column; height: 12rem; width: 10rem; align-items: center; justify-content: space-evenly; color: var(--mdc-theme-on-surface); background-color: var(--mdc-theme-surface); border-radius: 0.5rem; margin: 0.5rem;")}>
            <span class={css!("height: 1rem;")}> {&props.inner.name} </span>
            <img class={css!("width: 9rem; max-height: 9rem;")} src={format!("https://cdn.discordapp.com/emojis/{}.{}",props.inner.id, if props.inner.animated {"gif"} else {"png"})} />
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
