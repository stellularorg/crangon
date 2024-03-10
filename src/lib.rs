#![doc = include_str!("../README.md")]
#![doc(html_root_url = "https://stellular.net/api/docs/bundlrs/")]
#![doc(html_favicon_url = "https://stellular.net/static/favicon.svg")]
#![doc(
    html_logo_url = "https://code.stellular.org/repo-avatars/cc8d0efab0759fa6310b75fd5759c33169ee0ab354a958172ed4425a66d2593b"
)]
#![doc(issue_tracker_base_url = "https://code.stellular.org/stellular/bundlrs/issues/")]

pub mod api;
mod components;
mod config;
pub mod db;
pub mod markdown;
pub mod pages;
pub mod ssm;
mod utility;
