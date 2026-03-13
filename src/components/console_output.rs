use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::components::ui::button::Button;
use crate::components::ui::card::Card;

#[derive(Clone, Debug)]
pub struct ConsoleEntry {
    pub entry_type: ConsoleEntryType,
    pub message: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConsoleEntryType {
    Info,
    Data,
    Warning,
    Error,
}

impl ConsoleEntryType {
    pub fn color_class(&self) -> &'static str {
        match self {
            ConsoleEntryType::Info => "text-blue-400",
            ConsoleEntryType::Data => "text-green-400",
            ConsoleEntryType::Warning => "text-yellow-400",
            ConsoleEntryType::Error => "text-red-400",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ConsoleEntryType::Info => "INFO",
            ConsoleEntryType::Data => "DATA",
            ConsoleEntryType::Warning => "WARN",
            ConsoleEntryType::Error => "ERROR",
        }
    }
}

#[component]
pub fn ConsoleOutput(
    #[prop(into)] entries: Signal<Vec<ConsoleEntry>>,
    #[prop(into)] on_clear: Callback<()>,
) -> impl IntoView {
    let console_ref = NodeRef::<leptos::html::Div>::new();

    Effect::new(move || {
        let _ = entries.get();
        if let Some(el) = console_ref.get() {
            let el: web_sys::Element = el.unchecked_into();
            // Use requestAnimationFrame to scroll after DOM update
            let cb = Closure::once_into_js(move || {
                el.set_scroll_top(el.scroll_height());
            });
            let _ = leptos::web_sys::window()
                .unwrap()
                .request_animation_frame(cb.unchecked_ref());
        }
    });

    view! {
        <Card class="h-full flex flex-col p-0">
            // Header
            <div class="px-2 py-1.5 border-b border-slate-700/50 flex items-center justify-between">
                <h2 class="text-xs font-semibold text-white flex items-center gap-1.5">
                    <svg class="w-3.5 h-3.5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                    "Console Output"
                </h2>
                <Button
                    variant=crate::components::ui::button::ButtonVariant::Secondary
                    class="px-2 py-1 rounded text-[11px]"
                    on_click=on_clear
                >
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                    <span>"Clear"</span>
                </Button>
            </div>

            // Console area
            <div node_ref=console_ref class="flex-1 p-2 overflow-y-auto console-text text-[11px] space-y-0.5 min-h-0">
                {move || {
                    let data = entries.get();
                    if data.is_empty() {
                        view! {
                            <div class="flex items-center justify-center h-full text-slate-500">
                                <div class="text-center">
                                    <svg class="w-10 h-10 mx-auto mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                    </svg>
                                    <p class="text-xs">"Waiting for data..."</p>
                                    <p class="text-[10px] mt-1">"Connect to a serial port to start monitoring"</p>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                {data.into_iter().map(|entry| {
                                    let color = entry.entry_type.color_class();
                                    let label = entry.entry_type.label();
                                    view! {
                                        <div class="flex gap-2 hover:bg-slate-700/30 px-1.5 py-0.5 rounded">
                                            <span class="text-slate-500 shrink-0">
                                                {format!("[{}]", entry.timestamp)}
                                            </span>
                                            <span class=format!("{} shrink-0 font-semibold uppercase text-[10px]", color)>
                                                {label}
                                            </span>
                                            <span class="text-slate-300">{entry.message}</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </Card>
    }
}
