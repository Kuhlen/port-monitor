use leptos::prelude::*;

#[component]
pub fn Card(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class=format!(
            "bg-slate-800/50 backdrop-blur-sm rounded-lg p-2 border border-slate-700/50 shadow-xl {}",
            class
        )>
            {children()}
        </div>
    }
}
