use runs_inside_qemu::runs_inside_qemu;

fn main() {
    // If we are in QEMU, we use the nice "debugcon"-feature which maps
    // the x86 I/O-port `0xe9` to stdout or a file.
    if runs_inside_qemu().is_very_likely() {
        unsafe {
            x86::io::outb(0xe9, b'H');
            x86::io::outb(0xe9, b'e');
            x86::io::outb(0xe9, b'l');
            x86::io::outb(0xe9, b'l');
            x86::io::outb(0xe9, b'o');
            x86::io::outb(0xe9, b'\n');
        }
    }
}
