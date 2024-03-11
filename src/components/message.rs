use super::avatar::AvatarDisplay;
use crate::db::bundlesdb::{BoardPostLog, Log};
use yew::prelude::*;

#[derive(Properties, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MessageProps {
    pub post: Log,
    pub show_open: bool,
    pub pinned: bool,
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

    let pinned = (props.pinned == true) | (post.pinned.is_some() && post.pinned.unwrap() == true); // show pin icon even when post is not in pinned section

    // ...
    return html! {
        <div class={format!(
            "message {} {} round full flex g-4",
            if post.reply.is_some() { "reply" } else { "" },
            if pinned == true { "pinned" } else { "" }
        )} title={if post.tags.is_some() {
            post.tags.unwrap()
        } else {
            String::new()
        }}>
            {if post.author != "Anonymous" {
                html! {<AvatarDisplay size={65} username={post.author.clone()} />}
            } else {
                html! {}
            }}

            <div class="flex flex-column g-4 full">
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

                    <div class="flex g-4 flex-wrap align-center">
                        {if post.replies.is_some() && post.replies.unwrap() > 0 {
                            html! { <>
                                <span title="Reply Count">{&post.replies.unwrap()}</span>
                                <span>{"·"}</span>
                            </> }
                        } else {
                            html! {}
                        }}

                        {if pinned == true {
                            html! {
                                <div class="flex align-center" style="color: var(--primary);" title="Pinned Post">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-pin"><line x1="12" x2="12" y1="17" y2="22"/><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"/></svg>
                                </div>
                            }
                        } else {
                            html! {}
                        }}

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
                </div>

                <div>{content}</div>
            </div>
        </div>
    };
}
