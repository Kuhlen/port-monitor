// Windows 7 stand-in for the Win8+ API set used by `arboard` (via Slint).
// Only PathCchStripPrefix is imported; this mirrors its documented behavior.
#include <windows.h>
#include <string.h>
#include <wchar.h>

__declspec(dllexport) HRESULT WINAPI PathCchStripPrefix(PWSTR path, size_t capacity) {
    if (path == NULL || capacity == 0) {
        return E_INVALIDARG;
    }
    // "\\?\UNC\server\share" -> "\\server\share"
    if (wcsncmp(path, L"\\\\?\\UNC\\", 8) == 0) {
        size_t rest = wcslen(path + 8);
        memmove(path + 2, path + 8, (rest + 1) * sizeof(WCHAR));
        return S_OK;
    }
    // "\\?\C:\dir" -> "C:\dir"
    if (wcsncmp(path, L"\\\\?\\", 4) == 0 && path[4] != L'\0' && path[5] == L':') {
        size_t rest = wcslen(path + 4);
        memmove(path, path + 4, (rest + 1) * sizeof(WCHAR));
        return S_OK;
    }
    return S_FALSE;
}
