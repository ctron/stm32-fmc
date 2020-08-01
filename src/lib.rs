//! Hardware Abstraction Layer for STM32 Memory Controllers (FMC/FSMC)
//!
//!
//! # Implementation Guide
//!
//! You can use the functionality in this crate by implementing the
//! [`FmcPeripheral`](FmcPeripheral) trait. You should implement this trait for a
//! structure that:
//!
//! * Takes ownership of the `FMC`/`FSMC` peripheral
//! * Takes ownership of any structures / ZSTs related to the power or clock for the `FMC`/`FSMC` peripheral
//! * Contains the frequency of the `FMC`/`FSMC` source clock (usually HCLK)
//!
//! A basic structure:
//!
//! ```
//! pub struct FMC {
//!     source_clock: u32,
//!     // any other fields here...
//! }
//! ```
//!
//! An implementation of [`FmcPeripheral`](FmcPeripheral):
//!
//! ```rust
//! use stm32_fmc::FmcPeripheral;
//!
//! unsafe impl Sync for FMC {}
//! unsafe impl FmcPeripheral for FMC {
//!     const REGISTERS: *const () = stm32::FMC::ptr() as *const ();
//!
//!     fn enable(&mut self) {
//!         // Enable and reset the FMC/FSMC using the RCC registers
//!         // Typically RCC.AHBxEN and RCC.AHBxRST
//!     }
//!
//!     fn memory_controller_enable(&mut self) {
//!         // Only required if your part has an `FMCEN` bit
//!     }
//!
//!     fn source_clock_hz(&self) -> u32 {
//!         self.hclk
//!     }
//! }
//! ```
//!
//! In a HAL, you can allow users to construct your structure by implementing a
//! `new` method, or by making the fields public.
//!
//! ## Wrap constructor methods
//!
//! Each memory controller type ([`SDRAM`](Sdram), `NAND` (todo), ..) provides both
//! `new` and `new_unchecked` methods.
//!
//! For the convenience of users, you may want to wrap these with your `new` method,
//! so that each memory can be created from the peripheral in one step.
//!
//! ```
//! use stm32_fmc::{PinsSdram, Sdram, SdramChip, SdramPinSet, SdramTargetBank};
//!
//! impl FMC {
//!     /// A new SDRAM memory via the Flexible Memory Controller
//!     pub fn sdram<BANK: SdramPinSet, PINS: PinsSdram<BANK>, CHIP: SdramChip>(
//!         fmc: stm32::FMC,
//!         pins: PINS,
//!         chip: CHIP,
//!         clocks: &CoreClocks,
//!     ) -> Sdram<FMC, CHIP> {
//!         let fmc = Self::new(fmc, clocks);
//!         Sdram::new(fmc, pins, chip)
//!     }
//!
//!     /// A new SDRAM memory via the Flexible Memory Controller
//!     pub fn sdram_unchecked<CHIP: SdramChip, BANK: Into<SdramTargetBank>>(
//!         fmc: stm32::FMC,
//!         bank: BANK,
//!         chip: CHIP,
//!         clocks: &CoreClocks,
//!     ) -> Sdram<FMC, CHIP> {
//!         let fmc = Self::new(fmc, clocks);
//!         Sdram::new_unchecked(fmc, bank, chip)
//!     }
//! }
//! ```
//!
//! # Pin implementations
//!
//! In contrast with the `new_unchecked` methods, the `new` methods require the user
//! pass a tuple as the `pins` argument. In a HAL, you can mark which types are
//! suitable as follows:
//!
//! ```rust
//! impl stm32_fmc::A0 for gpiof::PF0<Alternate<AF12>> {}
//! // ...
//! ```
//!

#![no_std]
// rustc lints.
#![warn(
    bare_trait_objects,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_qualifications,
    unused_results
)]

#[macro_use]
mod macros;

mod fmc;
pub use fmc::*;

#[cfg(feature = "sdram")]
mod sdram;
#[cfg(feature = "sdram")]
pub use sdram::{PinsSdram, SdramPinSet, Sdram, SdramChip, SdramTargetBank};

/// Memory device definitions
pub mod devices;

mod ral;

/// A trait for device-specific FMC peripherals. Implement this to add support
/// for a new hardware platform. Peripherals that have this trait must have the
/// same register block as STM32 FMC peripherals.
pub unsafe trait FmcPeripheral: Send + Sync {
    /// Pointer to the register block
    const REGISTERS: *const ();

    /// Enables the FMC on its peripheral bus
    fn enable(&mut self);

    /// Enables the FMC memory controller (not always required)
    fn memory_controller_enable(&mut self) {}

    /// The frequency of the clock used as a source for the fmc_clk.
    ///
    /// F4/F7/G4: hclk
    /// H7: fmc_ker_ck
    fn source_clock_hz(&self) -> u32;
}
