// Windows 7 stand-in for the Win8+ WinRT error API used by the `windows` crate.
// RoOriginateErrorW only attaches debugger info; FALSE means "not reported".
#include <windows.h>

__declspec(dllexport) BOOL WINAPI RoOriginateErrorW(HRESULT error, UINT length, PCWSTR message) {
    (void)error;
    (void)length;
    (void)message;
    return FALSE;
}
