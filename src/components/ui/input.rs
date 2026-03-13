use leptos::prelude::*;

#[component]
pub fn Input(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_input: Callback<String>,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional, into)] input_type: Option<String>,
    #[prop(optional)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] min: Option<String>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let handle_input = move |ev| {
        on_input.run(event_target_value(&ev));
    };

    let input_type = input_type.unwrap_or_else(|| "text".to_string());

    view! {
        <input
            type=input_type
            class=format!(
                "w-full bg-slate-700/50 border border-slate-600 rounded px-2 py-1 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed {}",
                class
            )
            prop:value=move || value.get()
            on:input=handle_input
            placeholder=placeholder
            disabled=move || disabled.map(|d| d.get()).unwrap_or(false)
            min=min.unwrap_or_default()
        />
    }
}
