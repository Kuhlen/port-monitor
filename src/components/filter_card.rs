use leptos::prelude::*;

use crate::components::ui::card::Card;
use crate::components::ui::input::Input;
use crate::components::ui::toggle::Toggle;

#[component]
pub fn FilterCard(
    #[prop(into)] filter_enabled: Signal<bool>,
    #[prop(into)] set_filter_enabled: Callback<bool>,
    #[prop(into)] offset: Signal<String>,
    #[prop(into)] set_offset: Callback<String>,
    #[prop(into)] length: Signal<String>,
    #[prop(into)] set_length: Callback<String>,
    #[prop(into)] exclude_chars: Signal<String>,
    #[prop(into)] set_exclude_chars: Callback<String>,
) -> impl IntoView {
    let disabled_when_off = Signal::derive(move || !filter_enabled.get());

    view! {
        <Card>
            <div class="flex items-center justify-between mb-1.5">
                <h2 class="text-xs font-semibold text-white flex items-center gap-1.5">
                    <svg
                        class="w-3.5 h-3.5 text-orange-500"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                        />
                    </svg>
                    "Data Filter"
                </h2>
                <Toggle checked=filter_enabled on_change=set_filter_enabled />
            </div>

            <div class=move || {
                format!(
                    "space-y-1.5 transition-opacity {}",
                    if filter_enabled.get() { "opacity-100" } else { "opacity-50" },
                )
            }>
                // Offset + Length
                <div class="grid grid-cols-2 gap-2">
                    <div>
                        <label class="block text-[11px] font-medium text-slate-300 mb-0.5">
                            "Offset"
                        </label>
                        <Input
                            value=offset
                            on_input=set_offset
                            input_type="number".to_string()
                            placeholder="0"
                            min="0".to_string()
                            disabled=disabled_when_off
                        />
                    </div>
                    <div>
                        <label class="block text-[11px] font-medium text-slate-300 mb-0.5">
                            "Length"
                        </label>
                        <Input
                            value=length
                            on_input=set_length
                            input_type="number".to_string()
                            placeholder="All"
                            min="1".to_string()
                            disabled=disabled_when_off
                        />
                    </div>
                </div>

                // Exclude Characters
                <div>
                    <label class="block text-[11px] font-medium text-slate-300 mb-0.5">
                        "Exclude Characters"
                    </label>
                    <Input
                        value=exclude_chars
                        on_input=set_exclude_chars
                        placeholder="e.g., \\n\\r or ,%"
                        disabled=disabled_when_off
                    />
                    <p class="text-[10px] text-slate-500 mt-1">
                        "Separate multiple characters without spaces"
                    </p>
                </div>

            </div>
        </Card>
    }
}
