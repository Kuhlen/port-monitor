Port Monitor for Linux x86_64 (glibc 2.35 or newer).

Run ./port-monitor. To open serial ports without root, add your user to the
dialout group, then log out and in again:
  sudo usermod -aG dialout "$USER"