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

//! Can we make this automatic ?

extern crate core;
extern crate rand;
extern crate spin;


pub use rw_lock::*;
pub use arch::ARCH;
pub use helpers::random_reader_idx;

mod rw_lock;
mod util;
mod helpers;
mod arch;

//mod tests;
//mod tests_many;

