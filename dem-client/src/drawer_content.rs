use material_yew::MatIconButton;
use stylist::yew::*;
use yew::prelude::*;

use crate::{AppLink, Routes};

#[derive(Debug, Properties, PartialEq)]
pub struct DrawerContentProps {
    pub onclick: Callback<()>,
    pub guilds: Vec<dem_http::models::PartialGuildWithPermission>,
}

#[function_component(DrawerContent)]
pub fn drawer_content(DrawerContentProps { onclick, guilds }: &DrawerContentProps) -> Html {
    html! {
        <>
            <div onclick={onclick.clone().reform(|_| ())}>
                <MatIconButton icon="close" label="Close" />
            </div>

            {
                    guilds.iter().map(|g|
                        html!{<GuildListItem guild={g.clone()} onclick={onclick.clone()}/>}
                    ).collect::<Html>()
            }
        </>
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
