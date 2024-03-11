use yew::prelude::*;

#[derive(Properties, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AvatarProps {
    pub username: String,
    pub size: i32,
}

#[function_component]
pub fn AvatarDisplay(props: &AvatarProps) -> Html {
    html! {
        <img
            class="avatar"
            style={format!("--size: {}px;", props.size)}
            src={format!("/api/auth/users/{}/avatar", props.username)}
        />
    }
}
