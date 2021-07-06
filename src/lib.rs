/*
MIT License

Copyright (c) 2021 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Small `no_std`-lib that checks if the binary is running inside a QEMU virtual machine.
//! Only works on x86/x86_64 platform. There are no heap allocation required.
//!
//! Under the hood, this is a wrapper around the awesome crate https://crates.io/crates/raw-cpuid.

#![no_std]

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use raw_cpuid::{CpuId, Hypervisor};
use raw_cpuid::{ExtendedFunctionInfo, HypervisorInfo};

/// Returns if the code is running inside a QEMU virtual machine.
/// Only works on x86/x86_64 platform.
///
/// Doesn't panic and in case something strange happens, it returns
/// `false` in favor of a `Result`, because these errors are absolutely
/// unlikely.
///
/// ## Example Usage
///
/// ```rust
/// use runs_inside_qemu::runs_inside_qemu;
///
/// fn main() {
///     // If we are in QEMU, we use the nice "debugcon"-feature which maps
///     // the x86 I/O-port `0xe9` to stdout or a file.
///     if runs_inside_qemu() {
///         unsafe {
///             x86::io::outb(0xe9, b'H');
///             x86::io::outb(0xe9, b'e');
///             x86::io::outb(0xe9, b'l');
///             x86::io::outb(0xe9, b'l');
///             x86::io::outb(0xe9, b'o');
///             x86::io::outb(0xe9, b'\n');
///         }
///     }
/// }
/// ```
pub fn runs_inside_qemu() -> bool {
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    compile_error!("This crate only works on the x86/x86_64-platform.");

    let id = CpuId::new();

    // ########## STEP 1 ##########
    let feature_info = id.get_feature_info();
    if feature_info.is_none() {
        // should never happen, except the CPU or virtualization environment does weird things
        log::debug!(
            "feature_info is not present, this is REALLY strange. Can't verify if we are in QEMU."
        );
        return false;
    }
    let feature_info = feature_info.unwrap();
    if !feature_info.has_hypervisor() {
        // QEMU is a hypervisor and no real machine => exit if flag not set
        // If we run in a Hypervisor, this flag is set (also see https://lwn.net/Articles/301888/)
        log::debug!("Hypervisor-flag is not set, we are not in QEMU");
        return false;
    }

    // ########## STEP 2 ##########
    // Now we know that we are in a Hypervisor environment
    let hypervisor_info = id.get_hypervisor_info();
    if hypervisor_info.is_none() {
        // should never happen, except the CPU or virtualization environment does weird things
        log::debug!("hypervisor_info is not present but hypervisor-flag is, this is REALLY strange");
        return false;
    }
    let hypervisor_info = hypervisor_info.unwrap();
    // if this returns false, because the hypervisor ID can be "KVM",
    // we still could be executed by QEMU -> further checks needed
    if hypervisor_has_qemu_id(&hypervisor_info) {
        log::debug!("QEMU is the direct hypervisor");
        return true;
    }

    // ########## STEP 3 ##########
    // now check the extended CPU id, which is provided by QEMU
    let extended_info = id.get_extended_function_info();
    if extended_info.is_none() {
        return false;
    }
    let extended_info = extended_info.unwrap();
    let is_qemu = extended_brand_string_contains_qemu(&extended_info);
    if is_qemu {
        log::debug!("Runs inside QEMU with {:?} as accelerator", hypervisor_info.identify());
    }
    is_qemu
}

/// Checks if the Hypervisor-ID is the well-known value of QEMU.
/// If this fails, we still could be in a QEMU environment, because
/// if QEMU is accelerated by KVM, the Hypervisor-ID is the one from KVM.
fn hypervisor_has_qemu_id(info: &HypervisorInfo) -> bool {
    match info.identify() {
        // `TCGTCGTCGTCG` is the magic value of the CPU signature of QEMU,
        // see https://github.com/qemu/qemu/blob/6512fa497c2fa9751b9d774ab32d87a9764d1958/target/i386/cpu.c
        Hypervisor::Unknown(0x5447_4354, 0x4354_4743, 0x4743_5447) => {
            // definitely QEMU
            true
        }
        _ => false,
    }
}

/// Consumes the extended function info from CPU-ID. In a QEMU environment,
/// this contains a string such as *QEMU Virtual CPU version 2.5+*.
/// If this returns true, we are in QEMU. If not, we are not.
fn extended_brand_string_contains_qemu(info: &ExtendedFunctionInfo) -> bool {
    info.processor_brand_string()
        .filter(|s| s.contains("QEMU"))
        .map_or(false, |_| true)
}
