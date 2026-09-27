// Windows 7 stand-in for combase.dll (Win8+). All exports are forwarded to
// ole32.dll by combase.def; this file only provides the DLL entry point.
#include <windows.h>

BOOL WINAPI DllMain(HINSTANCE instance, DWORD reason, LPVOID reserved) {
    (void)instance;
    (void)reason;
    (void)reserved;
    return TRUE;
}
