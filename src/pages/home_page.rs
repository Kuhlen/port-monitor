use leptos::prelude::*;

use crate::components::connection_card::ConnectionCard;
use crate::components::console_output::{ConsoleEntry, ConsoleEntryType, ConsoleOutput};
use crate::components::filter_card::FilterCard;
use crate::components::ui::custom_select::SelectOption;
use crate::tauri_commands;

#[component]
pub fn HomePage() -> impl IntoView {
    // Connection settings
    let (port, set_port) = signal(String::new());
    let (baud_rate, set_baud_rate) = signal("9600".to_string());
    let (data_bits, set_data_bits) = signal("8".to_string());
    let (stop_bits, set_stop_bits) = signal("1".to_string());
    let (parity, set_parity) = signal("none".to_string());
    let (flow_control, set_flow_control) = signal("none".to_string());
    let (is_connected, set_is_connected) = signal(false);

    // Port options (dynamic from scan)
    let (port_options, set_port_options) = signal(Vec::<SelectOption>::new());

    // Filter settings
    let (filter_enabled, set_filter_enabled) = signal(false);
    let (offset, set_offset) = signal("0".to_string());
    let (length, set_length) = signal(String::new());
    let (exclude_chars, set_exclude_chars) = signal(String::new());

    // Console data
    let (entries, set_entries) = signal(Vec::<ConsoleEntry>::new());

    let add_entry = move |entry_type: ConsoleEntryType, message: String, timestamp: String| {
        set_entries.update(|e| {
            e.push(ConsoleEntry {
                entry_type,
                message,
                timestamp,
            });
        });
    };

    // Scan ports
    let on_scan = Callback::new(move |_: ()| {
        leptos::task::spawn_local(async move {
            match tauri_commands::list_ports().await {
                Ok(ports) => {
                    let options: Vec<SelectOption> = ports
                        .iter()
                        .map(|p| SelectOption::new(&p.name, &p.name))
                        .collect();

                    if let Some(first) = options.first() {
                        if port.get_untracked().is_empty() {
                            set_port.set(first.value.clone());
                        }
                    }

                    set_port_options.set(options);
                }
                Err(e) => {
                    let timestamp = get_timestamp();
                    add_entry(
                        ConsoleEntryType::Error,
                        format!("Scan failed: {e}"),
                        timestamp,
                    );
                }
            }
        });
    });

    // Connect/Disconnect
    let on_connect = Callback::new(move |_: ()| {
        let connected = is_connected.get_untracked();

        if connected {
            // Disconnect
            leptos::task::spawn_local(async move {
                match tauri_commands::disconnect_port().await {
                    Ok(()) => {
                        set_is_connected.set(false);
                        let timestamp = get_timestamp();
                        add_entry(
                            ConsoleEntryType::Warning,
                            "Disconnected from serial port".to_string(),
                            timestamp,
                        );
                    }
                    Err(e) => {
                        let timestamp = get_timestamp();
                        add_entry(
                            ConsoleEntryType::Error,
                            format!("Disconnect failed: {e}"),
                            timestamp,
                        );
                    }
                }
            });
        } else {
            // Connect
            let config = tauri_commands::SerialConfig {
                port: port.get_untracked(),
                baud_rate: baud_rate.get_untracked().parse().unwrap_or(9600),
                data_bits: data_bits.get_untracked(),
                stop_bits: stop_bits.get_untracked(),
                parity: parity.get_untracked(),
                flow_control: flow_control.get_untracked(),
            };

            let port_name = config.port.clone();
            let baud = config.baud_rate;

            leptos::task::spawn_local(async move {
                match tauri_commands::connect_port(&config).await {
                    Ok(()) => {
                        set_is_connected.set(true);
                        let timestamp = get_timestamp();
                        add_entry(
                            ConsoleEntryType::Info,
                            format!("Connected to {port_name} at {baud} baud"),
                            timestamp,
                        );
                    }
                    Err(e) => {
                        let timestamp = get_timestamp();
                        add_entry(
                            ConsoleEntryType::Error,
                            format!("Connection failed: {e}"),
                            timestamp,
                        );
                    }
                }
            });
        }
    });

    // Listen for serial data events
    tauri_commands::listen_serial_data(move |event| {
        let mut message = event.data.clone();

        // Apply filter if enabled
        if filter_enabled.get_untracked() {
            let offset_val: usize = offset.get_untracked().parse().unwrap_or(0);
            let length_val: Option<usize> = {
                let l = length.get_untracked();
                if l.is_empty() {
                    None
                } else {
                    l.parse().ok()
                }
            };
            let exclude = exclude_chars.get_untracked();

            // Apply offset and length
            if offset_val < message.len() {
                let end = length_val
                    .map(|l| (offset_val + l).min(message.len()))
                    .unwrap_or(message.len());
                message = message[offset_val..end].to_string();
            } else {
                return;
            }

            // Exclude characters
            if !exclude.is_empty() {
                for ch in exclude.chars() {
                    message = message.replace(ch, "");
                }
            }

            if message.is_empty() {
                return;
            }
        }

        add_entry(ConsoleEntryType::Data, message, event.timestamp);
    });

    // Listen for serial error events
    tauri_commands::listen_serial_error(move |event| {
        set_is_connected.set(false);
        add_entry(ConsoleEntryType::Error, event.message, event.timestamp);
    });

    let on_clear = Callback::new(move |_| {
        set_entries.set(Vec::new());
    });

    // Trigger initial scan
    on_scan.run(());

    view! {
        <div class="h-screen p-2 overflow-hidden">
            <div class="grid grid-cols-1 md:grid-cols-[1fr_3fr] gap-2 h-full">
                // Left column: Connection + Filter
                <div class="space-y-2 overflow-y-auto">
                    <ConnectionCard
                        port=Signal::derive(move || port.get())
                        set_port=Callback::new(move |v| set_port.set(v))
                        baud_rate=Signal::derive(move || baud_rate.get())
                        set_baud_rate=Callback::new(move |v| set_baud_rate.set(v))
                        data_bits=Signal::derive(move || data_bits.get())
                        set_data_bits=Callback::new(move |v| set_data_bits.set(v))
                        stop_bits=Signal::derive(move || stop_bits.get())
                        set_stop_bits=Callback::new(move |v| set_stop_bits.set(v))
                        parity=Signal::derive(move || parity.get())
                        set_parity=Callback::new(move |v| set_parity.set(v))
                        flow_control=Signal::derive(move || flow_control.get())
                        set_flow_control=Callback::new(move |v| set_flow_control.set(v))
                        is_connected=Signal::derive(move || is_connected.get())
                        port_options=Signal::derive(move || port_options.get())
                        on_connect=on_connect
                        on_scan=on_scan
                    />

                    <FilterCard
                        filter_enabled=Signal::derive(move || filter_enabled.get())
                        set_filter_enabled=Callback::new(move |v| set_filter_enabled.set(v))
                        offset=Signal::derive(move || offset.get())
                        set_offset=Callback::new(move |v| set_offset.set(v))
                        length=Signal::derive(move || length.get())
                        set_length=Callback::new(move |v| set_length.set(v))
                        exclude_chars=Signal::derive(move || exclude_chars.get())
                        set_exclude_chars=Callback::new(move |v| set_exclude_chars.set(v))
                    />

                </div>

                // Right column: Console Output
                <div class="min-h-0 h-full">
                    <ConsoleOutput
                        entries=Signal::derive(move || entries.get())
                        on_clear=on_clear
                    />
                </div>
            </div>
        </div>
    }
}

fn get_timestamp() -> String {
    let date = js_sys::Date::new_0();
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds(),
        date.get_milliseconds(),
    )
}
