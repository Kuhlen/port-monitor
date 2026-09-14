// The gate is REGISTRATION, not success. With no real serial device
// connect_port does fail, and that still passes. Only "command not found"
// fails the test: forgotten in handler!.
//
// Capabilities are not covered here: mock_context loads no real ACL, so
// stripping every permission stays green. The static check below guards that.
use tauri::ipc::CallbackFn;
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use port_monitor_lib::state::AppState;

const COMMANDS: &[(&str, &str)] = &[
    ("list_ports", "{}"),
    (
        "connect_port",
        r#"{"config":{"port":"/dev/null","baud_rate":9600,"data_bits":"8","stop_bits":"1","parity":"none","flow_control":"none"}}"#,
    ),
    ("disconnect_port", "{}"),
    ("check_update", "{}"),
    ("install_update", "{}"),
];

fn test_app() -> tauri::App<tauri::test::MockRuntime> {
    // Plugin list must mirror run(): a missing updater makes app.updater()
    // panic instead of failing cleanly.
    #[allow(unused_mut)]
    let mut builder = tauri::test::mock_builder().plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    let app = builder
        .invoke_handler(port_monitor_lib::handler!())
        .build(mock_context())
        .expect("failed to build test app");
    app.manage(AppState::new(app.handle().clone()));
    app
}

// The default mock_context carries no plugin config, and the updater refuses
// to initialize without it. Pull it from the real tauri.conf.json - which also
// guards against the plugins block being deleted.
fn mock_context() -> tauri::Context<tauri::test::MockRuntime> {
    let mut ctx = tauri::test::mock_context(tauri::test::noop_assets());
    let conf: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    ctx.config_mut().plugins =
        serde_json::from_value(conf["plugins"].clone()).expect("`plugins` block missing");
    ctx
}

#[test]
fn every_command_registered() {
    let app = test_app();
    let win = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
        .build()
        .unwrap();

    for (cmd, body) in COMMANDS {
        let res = tauri::test::get_ipc_response(
            &win,
            InvokeRequest {
                cmd: cmd.to_string(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: serde_json::from_str::<serde_json::Value>(body)
                    .unwrap()
                    .into(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        );

        if let Err(e) = &res {
            let msg = format!("{:?}", e);
            assert!(
                !msg.contains("not found") && !msg.contains("not allowed"),
                "command `{}` not registered in handler!: {}",
                cmd,
                msg
            );
        }
    }

    assert_eq!(COMMANDS.len(), 5, "command count changed - update the list");
}

// mock_context loads no ACL, so permissions are untestable over IPC. Static
// stand-in: the webview only needs core (invoke + listen). The updater is
// called from Rust, out of ACL reach.
#[test]
fn capabilities_declare_permissions() {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
    let perms: Vec<&str> = v["permissions"]
        .as_array()
        .expect("permissions must be an array")
        .iter()
        .map(|p| p.as_str().expect("permission must be a string"))
        .collect();

    assert!(
        perms.contains(&"core:default"),
        "capabilities/default.json is missing `core:default`"
    );
}
