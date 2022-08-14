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

mod drawer_content;
mod error;
mod style;
mod emoji_list;

use error::CloneError;

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

#[styled_component(App)]
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
        <style::MatThemeSetter ..style::MatThemeSetterProps::DARK_THEME/>
        <ContextProvider<APIConfig> context={(*config).clone()}>
            <HashRouter>
                <MatDrawer open={*drawer} drawer_type="dismissible">
                {
                    if let Some(guilds) = &guilds.data {
                        html! {<drawer_content::DrawerContent onclick={onclick.clone()} guilds={guilds.clone()} />}   
                    } else if let Some(error) = &guilds.error {
                        html!{<error::ErrorComponent name={error.catergory()} description={error.detail()} />}
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
                                html!{<error::ErrorComponent name={error.catergory()} description={error.detail()} />}
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
                    } else if let Some(error) = guilds.error.clone() {
                        html! {<error::ErrorComponent name={error.catergory()} description={error.detail()} />}
                    }else {
                        html!{"Loading..."}
                    }
                }
            </HashRouter>
        </ContextProvider<APIConfig>>
    </>
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
                    <emoji_list::GuildEmojiList {id} />
                }
            } else {
                html! {
                    <error::ErrorComponent description={"Guild not found"} name={"Invalid Url"} />
                }
            }
        }
    }
}


fn main() {
    yew::Renderer::<App>::new().render();
}
