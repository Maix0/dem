use yew::prelude::*;

#[function_component(MainStyle)]
fn style_component() -> Html {
    let style = r"
    html, body {
        padding: 0px;
        margin: 0px;
    }
    * {
        font-family: 'Roboto', sans-serif;
    }
    a {
        text-decoration: none;
    }
    .guild-list-item img {
        width: 3em;
        height: 3em;
        border-radius: 50%;
        padding-right: 1em;
    }

    .guild-list-item span {
        text-decoration: none;
        color: var(--mdc-theme-on-surface);
    }


    .guild-list-item {
        padding-top: 0px;
        margin-top: 0px;
        display: flex;
        flex-direction: row;
        flex-wrap: nowrap;
        justify-content: flex-start;
        align-items: center;
        height: 4em;
    }

    .emoji_item {
        display: flex;
        flex-direction: column;
        height: 12rem;
        width: 10rem;
        align-items: center;
        justify-content: space-evenly;
        color: var(--mdc-theme-on-surface);
        background-color: var(--mdc-theme-surface);
        border-radius: 0.5rem;
        margin: 0.5rem;
    }

    .emoji_item span {
        height: 1rem;
    }

    .emoji_item img {
        width: 9rem;
        max-height: 9rem;
    }

    .main_app {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        justify-content: space-between;
    }
    ";

    html! {<stylist::yew::Global css={style}/> }
}
