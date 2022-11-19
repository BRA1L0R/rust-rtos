use core::{
    mem::{size_of, MaybeUninit},
    pin::Pin,
};

extern crate alloc;
use alloc::boxed::Box;
use cortex_m::register::control::{Control, Npriv, Spsel};

use crate::TaskEntrypoint;

const STACK_SIZE: usize = 256;

pub type Stack = [MaybeUninit<u8>; STACK_SIZE];

#[derive(Default)]
#[repr(C)]
pub struct ExtendedFrame {
    // extended register content [40 bytes]
    /// control register
    control: u32,
    /// contains r4-r7
    low_regs: [u32; 4],
    /// contains r8-r12
    high_regs: [u32; 5],

    // ExceptionFrame
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
    lr: u32,
    pc: usize,
    xpsr: u32,
}

impl ExtendedFrame {
    fn control(self, control: Control) -> Self {
        Self {
            control: control.bits(),
            ..self
        }
    }

    fn pc(self, addr: usize) -> Self {
        Self { pc: addr, ..self }
    }

    fn thumb(self, active: bool) -> Self {
        Self {
            xpsr: self.xpsr | ((active as u32) << 24),
            ..self
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Task {
    stack: Pin<Box<Stack>>,
    /// offset from stack base,
    /// NOT absolute address
    pub(super) suspended_stack: *mut ExtendedFrame,
}

unsafe impl Send for Task {}

impl Task {
    pub fn create(entry: TaskEntrypoint) -> Self {
        // create a new uninitialized stack for the task
        let mut stack: Box<Stack> = unsafe { Box::new_uninit().assume_init() };

        const STACK_OFFSET: usize = STACK_SIZE - size_of::<ExtendedFrame>();
        let exception_frame = unsafe { stack.as_mut_ptr().add(STACK_OFFSET) } as *mut ExtendedFrame;

        let mut control = Control::from_bits(0);
        control.set_spsel(Spsel::Psp);
        control.set_npriv(Npriv::Unprivileged);

        unsafe {
            *exception_frame = ExtendedFrame::default()
                .pc(entry as usize)
                .thumb(true)
                .control(control)
        };

        Self {
            stack: Pin::new(stack),
            suspended_stack: exception_frame,
        }
    }

    pub fn sp(&self) -> *mut ExtendedFrame {
        self.suspended_stack
    }
}
