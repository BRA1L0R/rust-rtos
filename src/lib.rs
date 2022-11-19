#![no_std]
#![feature(naked_functions, new_uninit, asm_const)]

// use cortex_m::register::control::Control;

pub mod api;
pub(crate) mod arch;
mod scheduler;
pub mod supervisor;
mod syscall;

pub type TaskEntrypoint = extern "C" fn() -> !;

// unsafe fn modify_control(m: impl Fn(&mut Control)) {
//     use cortex_m::register::control;

//     let mut control_reg = control::read();
//     m(&mut control_reg);
//     control::write(control_reg);
// }
