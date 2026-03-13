use leptos::prelude::*;

use crate::tauri_commands;

#[component]
pub fn UpdateDialog() -> impl IntoView {
    let (update_available, set_update_available) = signal(false);
    let (update_version, set_update_version) = signal(String::new());
    let (is_installing, set_is_installing) = signal(false);

    // Check for updates on mount
    leptos::task::spawn_local(async move {
        if let Ok(Some(info)) = tauri_commands::check_for_update().await {
            set_update_version.set(info.version);
            set_update_available.set(true);
        }
    });

    let on_install = move |_| {
        set_is_installing.set(true);
        leptos::task::spawn_local(async move {
            match tauri_commands::install_update().await {
                Ok(()) => {
                    // App will restart automatically after install
                }
                Err(_) => {
                    set_is_installing.set(false);
                }
            }
        });
    };

    let on_dismiss = move |_| {
        set_update_available.set(false);
    };

    view! {
        <Show when=move || update_available.get()>
            <div class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center">
                <div class="bg-slate-800 border border-slate-600 rounded-lg p-4 shadow-xl max-w-sm mx-4">
                    <div class="flex items-center gap-2 mb-3">
                        <svg class="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                        </svg>
                        <h3 class="text-sm font-semibold text-white">"Update Available"</h3>
                    </div>
                    <p class="text-xs text-slate-300 mb-4">
                        "A new version "
                        <span class="font-semibold text-blue-400">{move || update_version.get()}</span>
                        " is available. Would you like to update now?"
                    </p>
                    <div class="flex gap-2 justify-end">
                        <button
                            class="px-3 py-1.5 text-xs rounded bg-slate-700 hover:bg-slate-600 text-slate-300 transition-colors"
                            on:click=on_dismiss
                            disabled=move || is_installing.get()
                        >
                            "Later"
                        </button>
                        <button
                            class="px-3 py-1.5 text-xs rounded bg-blue-600 hover:bg-blue-500 text-white transition-colors disabled:opacity-50"
                            on:click=on_install
                            disabled=move || is_installing.get()
                        >
                            {move || if is_installing.get() { "Installing..." } else { "Update Now" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
