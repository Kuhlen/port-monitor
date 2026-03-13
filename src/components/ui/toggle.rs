use leptos::prelude::*;
use leptos::web_sys;

#[component]
pub fn Toggle(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
) -> impl IntoView {
    let handle_change = move |ev: leptos::ev::Event| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);
        on_change.run(target.checked());
    };

    view! {
        <label class="relative inline-flex items-center cursor-pointer">
            <input
                type="checkbox"
                class="sr-only peer"
                prop:checked=move || checked.get()
                on:change=handle_change
            />
            <div class="w-8 h-4 bg-slate-700 peer-focus:outline-none peer-focus:ring-1 peer-focus:ring-orange-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3 after:w-3 after:transition-all peer-checked:bg-orange-500"></div>
        </label>
    }
}
