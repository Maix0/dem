use yew::prelude::*;

#[function_component(MainStyle)]
pub fn style_component() -> Html {
    let style = r"
    html, body {
        padding: 0px;
        margin: 0px;
    }
    *, a {
        font-family: 'Roboto', sans-serif;
        text-decoration: none;
    }";
    html! {<stylist::yew::Global css={style}/> }
}
