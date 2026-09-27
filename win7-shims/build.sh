#!/usr/bin/env bash
# Builds the Win7 shim DLLs (32-bit) that satisfy Win8+ imports inside Slint's
# dependencies. Windows 8+ ignores them: combase.dll is a KnownDLL and api-ms-*
# names resolve through the system API set schema.
# Usage: win7-shims/build.sh [output-dir]   (default: target/win7-shims)
set -euo pipefail

cd "$(dirname "$0")"
out="${1:-../target/win7-shims}"
mkdir -p "$out"
cc="${CC_I686_MINGW:-i686-w64-mingw32-gcc}"
flags=(-shared -O2 -s -static-libgcc)

# --kill-at exports stdcall functions undecorated (PathCchStripPrefix, not PathCchStripPrefix@8).
"$cc" "${flags[@]}" -Wl,--kill-at -o "$out/api-ms-win-core-path-l1-1-0.dll" api-ms-win-core-path-l1-1-0.c
"$cc" "${flags[@]}" -Wl,--kill-at -o "$out/api-ms-win-core-winrt-error-l1-1-0.dll" api-ms-win-core-winrt-error-l1-1-0.c
"$cc" "${flags[@]}" -o "$out/combase.dll" combase.c combase.def

echo "shims written to $out"
