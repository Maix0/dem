#[macro_use]
extern crate weblog;
extern crate bounce;
extern crate dem_http;
extern crate gloo_utils;
extern crate material_yew;
extern crate stylist;
extern crate yew;
extern crate yew_hooks;

use bounce::{prelude::*, query::*};
use material_yew::{top_app_bar_fixed::*, *};
use stylist::yew::*;
use yew::prelude::*;
use yew_router::prelude::*;

mod drawer_content;
mod emoji_list;
mod error;
mod style;
#[macro_use]
mod query;

use error::CloneError;

#[derive(Debug, Clone, Default, Atom)]
pub struct APIConfig(dem_http::apis::configuration::Configuration);

impl PartialEq<APIConfig> for APIConfig {
    fn eq(&self, _: &APIConfig) -> bool {
        true
    }
}

impl ::std::ops::Deref for APIConfig {
    type Target = dem_http::apis::configuration::Configuration;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

type AppLink = Link<Routes>;

#[styled_component(Root)]
fn root() -> Html {
    html! {
        <>
        <style::MainStyle />
        <style::MatThemeSetter ..style::MatThemeSetterProps::DARK_THEME/>
        <bounce::BounceRoot>
            <App />
        </bounce::BounceRoot>
        </>
    }
}

#[styled_component(App)]
fn app() -> Html {
    let drawer = use_state(|| false);
    let user_login = use_query_value::<query::CurrentUserQuery>(().into());
    let guilds = use_query_value::<query::UserGuildsQuery>(().into());

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
        <HashRouter>
            <MatDrawer open={*drawer} drawer_type="dismissible">
                <div class="drawer-content">
                    {
                        match guilds.result() {
                            None => Html::default(),
                            Some(Ok(guilds)) => html! {<drawer_content::DrawerContent onclick={onclick.clone()} guilds={(**guilds).clone()} />},
                            Some(Err(error)) => html!{<error::ErrorComponent name={error.catergory()} description={error.detail()} />},
                        }
                    }
                </div>
                <div slot="appContent">
                    <MatTopAppBar onnavigationiconclick={toggle_drawer}>
                        <MatTopAppBarNavigationIcon>
                            <MatIconButton icon="menu"></MatIconButton>
                        </MatTopAppBarNavigationIcon>


                        <MatTopAppBarTitle>
                            {"Discord Emojis Manager"}
                        </MatTopAppBarTitle>
                        <MatTopAppBarActionItems>
                            {
                                match user_login.result() {
                                    Some(Ok(o)) => match **o {
                                        Some(ref user) => html! {
                                            <div> {format!("Logged in as {}", user.username)} </div>
                                        },
                                        None =>
                                            html! {
                                                <a href="/api/auth">
                                                    <MatButton label="Login" icon={yew::virtual_dom::AttrValue::from("login")} unelevated=true trailing_icon=true/>
                                                </a>
                                            },
                                    },
                                    Some(Err(error)) => html!{<error::ErrorComponent name={error.catergory()} description={error.detail()} />},
                                    None =>
                                    html! {
                                        <a href="/api/auth">
                                            <MatButton label="Login" icon={yew::virtual_dom::AttrValue::from("login")} unelevated=true trailing_icon=true/>
                                        </a>
                                    },
                                }
                            }
                        </MatTopAppBarActionItems>

                    </MatTopAppBar>
                    <error::ErrorComponent name={"Dev Error".to_string()} description={"Test to see if it works".to_string()} />
                    {
                        match guilds.result() {
                            Some(Ok(guilds)) => {
                                html! {<Switch<Routes> render={Callback::<Routes, Html>::from(move |r| switch(&**guilds, r))} />}
                            },
                            Some(Err(error)) => {
                                html! {<error::ErrorComponent name={error.catergory()} description={error.detail()} />}
                            },
                            None => {
                                html!{"Loading..."}
                            }
                        }
                    }
                </div>
            </MatDrawer>
        </HashRouter>
    }
}

#[derive(Debug, Clone, PartialEq, Routable)]
enum Routes {
    #[at("/")]
    Main,
    #[at("/guild/:id")]
    Guild { id: u64 },
}

fn switch(guilds: &[dem_http::models::PartialGuildWithPermission], r: Routes) -> Html {
    match r {
        Routes::Main => html! {
            {"Main App"}
        },
        Routes::Guild { id } => {
            if guilds.iter().any(|g| g.id == id) {
                html! {
                    <>
                    <emoji_list::GuildEmojiList {id} />
                    <emoji_list::UploadedEmojiList {id} />
                    </>
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
    yew::Renderer::<Root>::new().render();
}
