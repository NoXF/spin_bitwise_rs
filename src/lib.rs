#![crate_type = "lib"]
#![cfg_attr(feature = "asm", feature(asm))]
//#![cfg_attr(feature = "core_intrinsics", feature(core_intrinsics))]
#![cfg_attr(feature = "const_fn", feature(const_fn))]
//#![warn(missing_docs)]

//#![no_std]

//#[cfg(test)]
#![feature(dec2flt)]

//#[macro_use]
//extern crate std;

//#![feature(const_fn)]



extern crate core;
extern crate rand;
extern crate spin;

pub use rw_lock::*;

mod rw_lock;
mod util;
mod helpers;
mod arch;

mod tests;
