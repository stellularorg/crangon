use crate::db::bundlesdb::{BoardPostLog, Log};
use yew::prelude::*;

#[derive(Properties, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MessageProps {
    pub post: Log,
    pub show_open: bool,
}

#[function_component]
pub fn Message(props: &MessageProps) -> Html {
    let p = &props.post;

    let post = serde_json::from_str::<BoardPostLog>(&p.content).unwrap();
    let content = Html::from_html_unchecked(AttrValue::from(
        post.content_html
            .clone()
            .replace("<style>", "&lt;style>")
            .replace("</style>", "&lt;/style>"),
    ));

    // ...
    return html! {
        <div class={format!("message {} round full flex flex-column g-4", if post.reply.is_some() { "reply" } else { "" })}>
            <div class="flex justify-space-between align-center g-4">
                <div class="flex g-4 flex-wrap">
                    <span class="chip mention round" style="width: max-content;">
                        {if post.author != "Anonymous" {
                            html! {<a href={format!("/~{}", &post.author)} style="color: inherit;">{&post.author}</a>}
                        } else {
                            html! {<span>{"Anonymous"}</span>}
                        }}
                    </span>

                    <span class="date-time-to-localize" style="opacity: 75%;">{&p.timestamp}</span>
                </div>

                {if props.show_open == true {
                    html! {
                        <div class="flex g-4 flex-wrap">
                            <a
                                class="button border round"
                                href={format!("/b/{}/posts/{}", post.board, p.id)}
                                style="color: var(--text-color);"
                                target="_blank"
                                title="open/manage"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-up-right-from-square"><path d="M21 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h6"/><path d="m21 3-9 9"/><path d="M15 3h6v6"/></svg>
                            </a>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>

            <div>{content}</div>
        </div>
    };
}
