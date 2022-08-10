#[macro_use]
extern crate weblog;
extern crate gloo_utils;
extern crate material_yew;
extern crate yew;

use std::ops::Deref;

use material_yew::{top_app_bar_fixed::*, *};
use yew::prelude::*;
use yew_router::prelude::*;
mod logged;
mod theme;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct UserLogin(dem_http::models::OkResponseForNullableUserLogin);

#[derive(Clone, Debug, PartialEq, Default)]
pub struct OverlappingGuilds(dem_http::models::OkResponseForArrayOfPartialGuildWithPermission);

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

#[function_component(App)]
fn app() -> Html {
    let user_login = use_state(UserLogin::default);
    let config = use_state(APIConfig::default);
    let guilds = use_state(OverlappingGuilds::default);
    let drawer = use_state(|| false);
    {
        use_effect_with_deps(
            {
                let user_login = user_login.clone();
                let config = config.clone();
                let guilds = guilds.clone();
                move |()| {
                    wasm_bindgen_futures::spawn_local(async move {
                        match dem_http::apis::default_api::api_get_current_user(&config).await {
                            Ok(u) => {
                                user_login.set(UserLogin(u));
                            }
                            Err(e) => console_error!(format!("{:?}", e)),
                        };
                        match dem_http::apis::default_api::api_get_overlapping_guilds(&config).await
                        {
                            Ok(g) => {
                                guilds.set(OverlappingGuilds(g));
                            }
                            Err(e) => console_error!(format!("{:?}", e)),
                        }
                    });
                    || ()
                }
            },
            (),
        )
    }

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
        <theme::MatThemeSetter ..theme::MatThemeSetterProps::DARK_THEME/>
        <ContextProvider<APIConfig> context={(*config).clone()}>
        <ContextProvider<UserLogin> context={(*user_login).clone()}>
        <ContextProvider<OverlappingGuilds> context={(*guilds).clone()}>
            <HashRouter>
                <MatDrawer open={*drawer} drawer_type="dismissible">
                    <div onclick={onclick.clone().reform(|_| ())}>
                        <MatIconButton icon="close" label="Close" />
                    </div>
                    {
                        guilds.0.ok.iter().map(|g|
                            html!{<GuildListItem guild={g.clone()} onclick={onclick.clone()}/>}
                        ).collect::<Html>()
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
                        <Login />
                    </MatTopAppBarActionItems>

                </MatTopAppBarFixed>
                <Switch<Routes> render={Callback::<Routes, Html>::from(move |r| switch(&*guilds, r))} />
            </HashRouter>
        </ContextProvider<OverlappingGuilds>>
        </ContextProvider<UserLogin>>
        </ContextProvider<APIConfig>>
    </>
    }
}

#[function_component(Login)]
fn login() -> Html {
    let logged_in = use_context::<UserLogin>().unwrap();
    if let Some(user) = logged_in.0.ok {
        html! {
            <div> {format!("Logged in as {}", user.username)} </div>
        }
    } else {
        html! {
            <a href="/api/auth">
                <MatButton label="Login" icon={yew::virtual_dom::AttrValue::from("login")} unelevated=true trailing_icon=true/>
            </a>
        }
    }
}

#[derive(Debug, Clone, Properties, PartialEq)]
struct Guild {
    guild: dem_http::models::PartialGuildWithPermission,
    onclick: Callback<()>,
}

#[function_component(GuildListItem)]
fn guild_list_item(
    Guild {
        guild: dem_http::models::PartialGuildWithPermission { id, name, icon, .. },
        onclick,
    }: &Guild,
) -> Html {
    html! {
        <AppLink to={Routes::Guild {id: *id} }>
        <div class={classes!["guild-list-item"]} onclick={onclick.reform(|_| ())}>
            <img src={yew::virtual_dom::AttrValue::from(format!("https://cdn.discordapp.com/icons/{id}/{icon}.png?size=1024"))} />
            <span>{name}</span>
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

fn switch(guilds: &OverlappingGuilds, r: Routes) -> Html {
    match r {
        Routes::Main => html! {
            <div class="main-app">
                {"Main App"}
            </div>
        },
        Routes::Guild { id } => {
            if guilds.0.ok.iter().any(|g| g.id == id) {
                html! {
                    <div class="main-app">
                        <GuildEmojiList {id} />
                    </div>
                }
            } else {
                html! {
                    <div class="main-app">
                        {"No guild found"}
                    </div>
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct EmojiListProps {
    id: u64,
}

#[function_component(GuildEmojiList)]
fn emoji_list(props: &EmojiListProps) -> Html {
    let api_config = use_context::<APIConfig>().unwrap();
    let emojis = use_state(Vec::<dem_http::models::EmojiItem>::new);

    use_effect_with_deps(
        {
            let id = props.id;
            let emojis = emojis.clone();
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    match dem_http::apis::default_api::api_get_guild_emojis(&api_config, id).await {
                        Ok(o) => emojis.set(o.ok),
                        Err(e) => console_error!("Error with DEM API", e.to_string()),
                    };
                });
                || ()
            }
        },
        (),
    );
    html! {
        {
            emojis.iter().map(|e| html! {
                <EmojiListItem inner={e.clone()}/>
            }
            ).collect::<Html>()
        }
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
struct EmojiListItemProps {
    pub inner: dem_http::models::EmojiItem,
}

#[function_component(EmojiListItem)]
fn emoji_list_item(props: &EmojiListItemProps) -> Html {
    html! {
        <div class={classes!["emoji_item"]}>
            <img src={format!("https://cdn.discordapp.com/emojis/{}.{}",props.inner.id, if props.inner.animated {"gif"} else {"png"})} />
            <span> {&props.inner.name} </span>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
