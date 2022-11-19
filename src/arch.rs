#[cfg(feature = "cortex-m0+")]
mod cortex_m0p;
#[cfg(feature = "cortex-m0+")]
use cortex_m0p as arch_impl;

pub(crate) use arch_impl::*;
