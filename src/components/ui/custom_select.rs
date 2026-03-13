use leptos::portal::Portal;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;

#[derive(Clone, Debug)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

impl SelectOption {
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
        }
    }
}

#[component]
pub fn CustomSelect(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<String>,
    #[prop(into)] options: Vec<SelectOption>,
    #[prop(optional)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let (show_dropdown, set_show_dropdown) = signal(false);
    let (dropdown_style, set_dropdown_style) = signal(String::new());
    let container_ref = NodeRef::<leptos::html::Div>::new();
    let button_ref = NodeRef::<leptos::html::Button>::new();
    let dropdown_ref = NodeRef::<leptos::html::Div>::new();
    let options_for_label = options.clone();
    let options_for_list = StoredValue::new(options.clone());

    let selected_label = move || {
        let current = value.get();
        options_for_label
            .iter()
            .find(|o| o.value == current)
            .map(|o| o.label.clone())
            .unwrap_or_else(|| "Select...".to_string())
    };

    let is_disabled = move || disabled.map(|d| d.get()).unwrap_or(false);

    let toggle = move |_| {
        if !is_disabled() {
            if !show_dropdown.get_untracked() {
                if let Some(btn) = button_ref.get() {
                    let el: &web_sys::HtmlElement = btn.as_ref();
                    let rect = js_sys::Reflect::get(&el.into(), &"getBoundingClientRect".into())
                        .unwrap()
                        .unchecked_into::<js_sys::Function>()
                        .call0(&el.into())
                        .unwrap();
                    let top = js_sys::Reflect::get(&rect, &"bottom".into())
                        .unwrap()
                        .as_f64()
                        .unwrap()
                        + 4.0;
                    let left = js_sys::Reflect::get(&rect, &"left".into())
                        .unwrap()
                        .as_f64()
                        .unwrap();
                    let width = js_sys::Reflect::get(&rect, &"width".into())
                        .unwrap()
                        .as_f64()
                        .unwrap();
                    set_dropdown_style.set(format!(
                        "position:fixed;top:{}px;left:{}px;width:{}px;z-index:9999;",
                        top, left, width
                    ));
                }
            }
            set_show_dropdown.update(|v| *v = !*v);
        }
    };

    // Close on click outside (check both container and dropdown portal)
    Effect::new(move |_| {
        use leptos::wasm_bindgen::prelude::Closure;

        let document = web_sys::window().unwrap().document().unwrap();

        let click_handler =
            Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
                if !show_dropdown.get_untracked() {
                    return;
                }
                if let Some(target) = ev.target() {
                    let target_node: &web_sys::Node = target.unchecked_ref();
                    // Check if click is inside the trigger container
                    let in_container = container_ref
                        .get()
                        .map(|c| c.contains(Some(target_node)))
                        .unwrap_or(false);
                    // Check if click is inside the dropdown portal
                    let in_dropdown = dropdown_ref
                        .get()
                        .map(|d| d.contains(Some(target_node)))
                        .unwrap_or(false);
                    if !in_container && !in_dropdown {
                        set_show_dropdown.set(false);
                    }
                }
            });

        let keydown_handler =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
                if ev.key() == "Escape" && show_dropdown.get_untracked() {
                    ev.prevent_default();
                    set_show_dropdown.set(false);
                }
            });

        document
            .add_event_listener_with_callback("click", click_handler.as_ref().unchecked_ref())
            .unwrap();
        document
            .add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref())
            .unwrap();

        click_handler.forget();
        keydown_handler.forget();
    });

    view! {
        <div
            node_ref=container_ref
            class=format!("relative w-full {}", class)
        >
            // Trigger button
            <button
                type="button"
                node_ref=button_ref
                on:click=toggle
                disabled=is_disabled
                class="flex w-full items-center justify-between bg-slate-700/50 border border-slate-600 rounded px-2 py-1 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
            >
                <span class="truncate">{selected_label}</span>
                <svg
                    class=move || format!(
                        "w-3 h-3 ml-1 shrink-0 transition-transform duration-200 text-slate-400 {}",
                        if show_dropdown.get() { "rotate-180" } else { "" }
                    )
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
            </button>
        </div>

        // Dropdown portal (rendered outside container, directly under body via Portal)
        <Portal mount=document().body().unwrap()>
            <Show when=move || show_dropdown.get()>
                <div
                    node_ref=dropdown_ref
                    class="max-h-48 overflow-auto rounded bg-slate-800 border border-slate-600 shadow-lg"
                    style=move || dropdown_style.get()
                >
                    {options_for_list.get_value()
                        .into_iter()
                        .map(|opt| {
                            let opt_value = opt.value.clone();
                            let opt_value2 = opt.value.clone();
                            view! {
                                <button
                                    type="button"
                                    class=move || format!(
                                        "w-full px-2 py-1 text-left text-xs transition-colors cursor-pointer {}",
                                        if value.get() == opt_value2 {
                                            "bg-blue-500/20 text-blue-300 font-semibold"
                                        } else {
                                            "text-slate-300 hover:bg-slate-700"
                                        }
                                    )
                                    on:click=move |_| {
                                        on_change.run(opt_value.clone());
                                        set_show_dropdown.set(false);
                                    }
                                >
                                    {opt.label}
                                </button>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
            </Show>
        </Portal>
    }
}
