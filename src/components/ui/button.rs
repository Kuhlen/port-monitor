use leptos::prelude::*;

#[derive(Default, Clone, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Danger,
    Secondary,
}

#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: Option<Signal<bool>>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let base = "text-xs font-semibold transition-all duration-200 flex items-center justify-center gap-1 cursor-pointer";

    let variant_class = match variant {
        ButtonVariant::Primary => {
            "bg-blue-500 hover:bg-blue-600 text-white shadow-lg shadow-blue-500/30"
        }
        ButtonVariant::Danger => {
            "bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/30"
        }
        ButtonVariant::Secondary => "bg-slate-700 hover:bg-slate-600 text-white",
    };

    let handle_click = move |_| {
        if let Some(cb) = &on_click {
            cb.run(());
        }
    };

    view! {
        <button
            class=format!("{} {} {}", base, variant_class, class)
            disabled=move || disabled.map(|d| d.get()).unwrap_or(false)
            on:click=handle_click
        >
            {children()}
        </button>
    }
}
