use leptos::prelude::*;

use crate::components::update_dialog::UpdateDialog;
use crate::pages::home_page::HomePage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <UpdateDialog />
        <HomePage />
    }
}
