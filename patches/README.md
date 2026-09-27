# Patched dependencies

Crates copied from crates.io and changed so the app loads on Windows 7.
Re-apply these patches whenever the Slint version changes.

| Crate | Version | Change |
|---|---|---|
| `i-slint-backend-winit` | 1.18.1 | `lib.rs`: `SystemParametersInfoForDpi` (Windows 10 1607+) is resolved at runtime with `GetProcAddress` instead of a static import; `Cargo.toml`: adds the `Win32_System_LibraryLoader` feature of `windows`. Search for "Win7 patch". |
| `i-slint-backend-winit` | 1.18.1 | `renderer/sw.rs`: on Windows always `present()` the full buffer instead of `present_with_damage`, because without DWM composition Windows discards window contents and Slint would leave stale pixels after resize/occlusion. Search for "Win7 patch". |

Win8+ DLLs that are imported but not needed at runtime on Win7 are satisfied by the shim DLLs in `win7-shims/` instead.
