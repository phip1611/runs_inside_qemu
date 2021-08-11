# Rust lib *runs_inside_qemu*

`runs_inside_qemu` is a small `no_std`-lib that checks if the binary is running inside a 
QEMU virtual machine. It doesn't need heap allocations and only works on `x86`/`x86_64` platform.

Under the hood, this is a wrapper around the awesome crate https://crates.io/crates/raw-cpuid.

This crate was build/tested with `rustc 1.55.0-nightly`. It won't work on stable 
as long as inline assembly is not part of stable rust.

## Example Code
```rust
use runs_inside_qemu::runs_inside_qemu;

fn main() {
    // If we are in QEMU, we use the nice "debugcon"-feature which maps
    // the x86 I/O-port `0xe9` to stdout or a file.
    if runs_inside_qemu() {
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
```

## Limitations
This doesn't work if you pass `-cpu host` to QEMU, because in this case the CPU brand string is 
not "QEMU Virtual CPU version 2.5+".
