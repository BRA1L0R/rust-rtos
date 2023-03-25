#[cfg(feature = "cortex-m0+")]
mod cortex_m0p;
#[cfg(feature = "cortex-m0+")]
use cortex_m0p as arch_impl;

#[cfg(feature = "cortex-m4")]
mod cortex_m4;
#[cfg(feature = "cortex-m4")]
use cortex_m4 as arch_impl;

mod common;

pub use arch_impl::*;
pub use common::*;
