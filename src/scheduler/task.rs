use core::{
    mem::{align_of, size_of, MaybeUninit},
    pin::Pin,
};

extern crate alloc;
use alloc::boxed::Box;
use cortex_m::register::control::{Control, Npriv, Spsel};

use crate::TaskEntrypoint;

const STACK_SIZE: usize = 256;

#[repr(C)]
#[repr(align(4))]
#[derive(Debug)]
pub struct Stack([MaybeUninit<u8>; STACK_SIZE]);

impl core::ops::Deref for Stack {
    type Target = [MaybeUninit<u8>; STACK_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Stack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default)]
#[repr(C)]
struct ExtendedFrame {
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

/// Opaque type representing an aligned
/// frame ready to be loaded
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct TaskFrame(*mut ExtendedFrame);

#[derive(Debug)]
#[repr(C)]
pub struct Task {
    stack: Pin<Box<Stack>>,
    /// offset from stack base,
    /// NOT absolute address
    pub(super) suspended_stack: TaskFrame,
}

unsafe impl Send for Task {}

impl Task {
    pub fn create(entry: TaskEntrypoint) -> Self {
        // create a new uninitialized stack for the task
        // safety: assuming init a MaybeUninit<u8> array
        let mut stack: Box<Stack> = unsafe { Box::new_uninit().assume_init() };
        const STACK_OFFSET: usize = STACK_SIZE - size_of::<ExtendedFrame>();

        // Safety: safety of alignment to be verified
        debug_assert_eq!(align_of::<Stack>(), align_of::<ExtendedFrame>());
        let exception_frame = unsafe { stack.as_mut_ptr().add(STACK_OFFSET) } as *mut ExtendedFrame;

        let mut control = Control::from_bits(0);
        control.set_spsel(Spsel::Psp);
        control.set_npriv(Npriv::Unprivileged);

        // Safety: accessing a succesfully allocated memory
        // space, with a correctly aligned pointer
        unsafe {
            *exception_frame = ExtendedFrame::default()
                .pc(entry as usize)
                .thumb(true)
                .control(control)
        };

        Self {
            stack: Pin::new(stack),
            suspended_stack: TaskFrame(exception_frame),
        }
    }

    pub fn sp(&self) -> TaskFrame {
        self.suspended_stack
    }
}
