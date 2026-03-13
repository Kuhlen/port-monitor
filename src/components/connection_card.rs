use leptos::prelude::*;

use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::Card;
use crate::components::ui::custom_select::{CustomSelect, SelectOption};

#[component]
pub fn ConnectionCard(
    #[prop(into)] port: Signal<String>,
    #[prop(into)] set_port: Callback<String>,
    #[prop(into)] baud_rate: Signal<String>,
    #[prop(into)] set_baud_rate: Callback<String>,
    #[prop(into)] data_bits: Signal<String>,
    #[prop(into)] set_data_bits: Callback<String>,
    #[prop(into)] stop_bits: Signal<String>,
    #[prop(into)] set_stop_bits: Callback<String>,
    #[prop(into)] parity: Signal<String>,
    #[prop(into)] set_parity: Callback<String>,
    #[prop(into)] flow_control: Signal<String>,
    #[prop(into)] set_flow_control: Callback<String>,
    #[prop(into)] is_connected: Signal<bool>,
    #[prop(into)] port_options: Signal<Vec<SelectOption>>,
    #[prop(into)] on_connect: Callback<()>,
    #[prop(into)] on_scan: Callback<()>,
) -> impl IntoView {
    let baud_options = vec![
        SelectOption::new("50", "50"),
        SelectOption::new("75", "75"),
        SelectOption::new("110", "110"),
        SelectOption::new("134", "134"),
        SelectOption::new("150", "150"),
        SelectOption::new("200", "200"),
        SelectOption::new("300", "300"),
        SelectOption::new("600", "600"),
        SelectOption::new("1200", "1200"),
        SelectOption::new("1800", "1800"),
        SelectOption::new("2400", "2400"),
        SelectOption::new("4800", "4800"),
        SelectOption::new("9600", "9600"),
        SelectOption::new("19200", "19200"),
        SelectOption::new("28800", "28800"),
        SelectOption::new("38400", "38400"),
        SelectOption::new("57600", "57600"),
        SelectOption::new("76800", "76800"),
        SelectOption::new("115200", "115200"),
        SelectOption::new("230400", "230400"),
        SelectOption::new("460800", "460800"),
        SelectOption::new("576000", "576000"),
        SelectOption::new("921600", "921600"),
    ];

    let data_bits_options = vec![
        SelectOption::new("5", "5"),
        SelectOption::new("6", "6"),
        SelectOption::new("7", "7"),
        SelectOption::new("8", "8"),
    ];

    let stop_bits_options = vec![
        SelectOption::new("1", "1"),
        SelectOption::new("1.5", "1.5"),
        SelectOption::new("2", "2"),
    ];

    let parity_options = vec![
        SelectOption::new("none", "None"),
        SelectOption::new("even", "Even"),
        SelectOption::new("odd", "Odd"),
        SelectOption::new("mark", "Mark"),
        SelectOption::new("space", "Space"),
    ];

    let flow_control_options = vec![
        SelectOption::new("none", "None"),
        SelectOption::new("hardware", "Hardware (RTS/CTS)"),
        SelectOption::new("software", "Software (XON/XOFF)"),
    ];

    let disabled_when_connected = Signal::derive(move || is_connected.get());
    let (is_scanning, set_is_scanning) = signal(false);

    let handle_scan = Callback::new(move |_: ()| {
        set_is_scanning.set(true);
        on_scan.run(());
        // Auto-reset after 1s
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(1000).await;
            set_is_scanning.set(false);
        });
    });

    view! {
        <Card>
            <h2 class="text-xs font-semibold text-white mb-1.5 flex items-center gap-1.5">
                <svg
                    class="w-3.5 h-3.5 text-blue-500"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                    />
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                    />
                </svg>
                "Connection"
            </h2>

            <div class="space-y-1.5">
                // Port
                <div>
                    <label class="block text-[11px] font-medium text-slate-300 mb-0.5">
                        "Port"
                    </label>
                    <div class="flex gap-1">
                        <div class="min-w-0 flex-1">
                            {move || {
                                let opts = port_options.get();
                                view! {
                                    <CustomSelect
                                        value=port
                                        on_change=set_port
                                        options=opts
                                        disabled=disabled_when_connected
                                    />
                                }
                            }}
                        </div>
                        <Button
                            variant=ButtonVariant::Secondary
                            class="px-1.5 py-1 rounded shrink-0"
                            on_click=handle_scan
                            disabled=disabled_when_connected
                        >
                            <svg
                                class=move || format!(
                                    "w-3 h-3 transition-transform {}",
                                    if is_scanning.get() { "animate-spin" } else { "" }
                                )
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                                />
                            </svg>
                        </Button>
                    </div>
                </div>

                // Baud Rate
                <div>
                    <label class="block text-[11px] font-medium text-slate-300 mb-0.5">
                        "Baud Rate"
                    </label>
                    <CustomSelect
                        value=baud_rate
                        on_change=set_baud_rate
                        options=baud_options
                        disabled=disabled_when_connected
                    />
                </div>

                // Data Bits + Stop Bits
                <div class="grid grid-cols-2 gap-2">
                    <div>
                        <label class="block text-[11px] font-medium text-slate-300 mb-0.5">
                            "Data Bits"
                        </label>
                        <CustomSelect
                            value=data_bits
                            on_change=set_data_bits
                            options=data_bits_options
                            disabled=disabled_when_connected
                        />
                    </div>
                    <div>
                        <label class="block text-[11px] font-medium text-slate-300 mb-0.5">
                            "Stop Bits"
                        </label>
                        <CustomSelect
                            value=stop_bits
                            on_change=set_stop_bits
                            options=stop_bits_options
                            disabled=disabled_when_connected
                        />
                    </div>
                </div>

                // Parity
                <div>
                    <label class="block text-[11px] font-medium text-slate-300 mb-0.5">
                        "Parity"
                    </label>
                    <CustomSelect
                        value=parity
                        on_change=set_parity
                        options=parity_options
                        disabled=disabled_when_connected
                    />
                </div>

                // Flow Control
                <div>
                    <label class="block text-[11px] font-medium text-slate-300 mb-0.5">
                        "Flow Control"
                    </label>
                    <CustomSelect
                        value=flow_control
                        on_change=set_flow_control
                        options=flow_control_options
                        disabled=disabled_when_connected
                    />
                </div>

                // Connect/Disconnect Button
                {move || {
                    let variant = if is_connected.get() {
                        ButtonVariant::Danger
                    } else {
                        ButtonVariant::Primary
                    };
                    if is_connected.get() {
                        view! {
                            <Button
                                variant=variant
                                class="w-full py-1.5 rounded"
                                on_click=on_connect
                            >
                                <svg
                                    class="w-3.5 h-3.5"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M6 18L18 6M6 6l12 12"
                                    />
                                </svg>
                                <span>"Disconnect"</span>
                            </Button>
                        }
                            .into_any()
                    } else {
                        view! {
                            <Button
                                variant=variant
                                class="w-full py-1.5 rounded"
                                on_click=on_connect
                            >
                                <svg
                                    class="w-3.5 h-3.5"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M13 10V3L4 14h7v7l9-11h-7z"
                                    />
                                </svg>
                                <span>"Connect"</span>
                            </Button>
                        }
                            .into_any()
                    }
                }}
            </div>

            // Status Indicator
            <div class="mt-1.5 pt-1.5 border-t border-slate-700">
                <div class="flex items-center justify-between">
                    <span class="text-slate-400 text-[11px]">"Status"</span>
                    <div class="flex items-center gap-1.5">
                        <div class=move || {
                            format!(
                                "w-2 h-2 rounded-full {}",
                                if is_connected.get() {
                                    "bg-green-500 animate-pulse-dot"
                                } else {
                                    "bg-slate-600"
                                },
                            )
                        }></div>
                        <span class=move || {
                            format!(
                                "text-[11px] font-medium {}",
                                if is_connected.get() { "text-green-400" } else { "text-slate-500" },
                            )
                        }>
                            {move || if is_connected.get() { "Connected" } else { "Disconnected" }}
                        </span>
                    </div>
                </div>
            </div>
        </Card>
    }
}
