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
//! Under the hood, this is a wrapper around the awesome crate <https://crates.io/crates/raw-cpuid>.

#![no_std]
#![deny(clippy::all)]
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!("This crate only works on the x86/x86_64-platform.");

use raw_cpuid::{CpuId, Hypervisor};

/// Result of [`runs_inside_qemu`] that tells with what certainty the code runs inside QEMU.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum QemuCertainty {
    /// The code definitely doesn't run inside QEMU, because the Hypervisor-flag is not set.
    DefinitelyNot,

    /// The code maybe runs inside QEMU. This is returned
    /// if the application runs inside KVM as hypervisor, but
    /// the CPU is not QEMU. If you know, that your development
    /// setup only uses QEMU/KVM as Hypervisor, this is equal
    /// to [`Self::VeryLikely`].
    ///
    /// This happens, if you use something like "-cpu host" instead of "-cpu qemu64".
    Maybe,

    /// The information obtained by CPUID clearly states, that the code runs inside QEMU.
    /// Be aware, that Hypervisors could implement similar behaviour as QEMU does.
    ///
    /// `High` is the case, if the Hypervisor ID is the one from QEMU or if the Hypervisor-ID
    /// is `KVM` and the ID of the CPU brand string contains `QEMU`.
    VeryLikely,
}

impl QemuCertainty {
    /// Returns if certainty is low/definitely not.
    pub fn is_definitely_not(self) -> bool {
        self == Self::DefinitelyNot
    }

    /// Returns if certainty is high/very likely.
    pub fn is_very_likely(self) -> bool {
        self == Self::VeryLikely
    }

    /// Returns if certainty is maybe or high.
    pub fn is_maybe_or_very_likely(self) -> bool {
        self == Self::Maybe || self == Self::VeryLikely
    }
}

/// Returns if the code is running inside a QEMU virtual machine.
/// See [`QemuCertainty`], which is the return type.
///
/// ## Example Usage
///
/// ```rust
/// # use runs_inside_qemu::runs_inside_qemu;
///
/// fn main() {
///     // If we are in QEMU, we use the nice "debugcon"-feature which maps
///     // the x86 I/O-port `0xe9` to stdout or a file.
///     if runs_inside_qemu().is_maybe_or_very_likely() {
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
pub fn runs_inside_qemu() -> QemuCertainty {
    let id = CpuId::new();

    // ########## CHECK 1 ##########
    // The `x86` library first checks if the Hypervisor flag is present in the `cpuid` features.
    // If yes, it reads the Hypervisor info leaf from `cpuid`.
    // Also see https://lwn.net/Articles/301888/)
    let hypervisor_info = id.get_hypervisor_info();
    if hypervisor_info.is_none() {
        // QEMU is a Hypervisor and no real machine => exit if this is None
        log::debug!(
            "Definitely not QEMU. Hypervisor flag is not set, no hypervisor info available."
        );
        return QemuCertainty::DefinitelyNot;
    }
    let hypervisor_info = hypervisor_info.unwrap();

    // if this returns false, because the hypervisor ID can be "KVM",
    // we still could be executed by QEMU -> further checks needed
    if matches!(hypervisor_info.identify(), Hypervisor::QEMU) {
        log::debug!("Runs very likely in QEMU. QEMU is the direct hypervisor (no KVM etc.).");
        return QemuCertainty::VeryLikely;
    }

    // ########## CHECK 2 ##########
    // now check the extended CPU brand string (which is specific for QEMU)
    let brand_string = id.get_processor_brand_string();
    if brand_string.is_none() {
        log::debug!(
            "Maybe QEMU. CPU brand string not available, can't verify if code runs inside QEMU."
        );
        return QemuCertainty::Maybe;
    }
    let brand_string = brand_string.unwrap();
    let brand_string = brand_string.as_str();

    let cpu_brand_string_contains_qemu = brand_string.contains("QEMU");
    if cpu_brand_string_contains_qemu {
        // "QEMU Virtual CPU version 2.5+"
        log::debug!(
            "Runs very likely in QEMU with {:?} as accelerator.",
            hypervisor_info.identify()
        );
        QemuCertainty::VeryLikely
    } else {
        log::debug!(
            "Maybe QEMU. Hypervisor is {:?} but CPU brand string is not the one from QEMU.",
            hypervisor_info.identify()
        );
        QemuCertainty::Maybe
    }
}
