extern crate alloc;

use crate::TaskEntrypoint;

use alloc::boxed::Box;
use core::{
    mem::{align_of, size_of, MaybeUninit},
    pin::Pin,
};
use cortex_m::register::control::{Control, Npriv, Spsel};

use super::arguments::AbiArguments;

const STACK_SIZE: usize = 512;

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
pub struct ExtendedFrame {
    // extended register content [40 bytes]
    /// control register
    control: u32,
    /// contains r4-r7
    low_regs: [u32; 4],
    /// contains r8-r12
    high_regs: [u32; 5],

    // ExceptionFrame
    abi_regs: [u32; 4], // r0-r3
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
pub struct FramePtr(*mut ExtendedFrame);

impl FramePtr {
    pub unsafe fn new(frame: *mut ExtendedFrame) -> Self {
        Self(frame)
    }
}

impl AsRef<ExtendedFrame> for FramePtr {
    fn as_ref(&self) -> &ExtendedFrame {
        unsafe { self.0.as_ref() }.unwrap()
    }
}

impl AsMut<ExtendedFrame> for FramePtr {
    fn as_mut(&mut self) -> &mut ExtendedFrame {
        unsafe { self.0.as_mut() }.unwrap()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Task {
    stack: Pin<Box<Stack>>,
    /// offset from stack base,
    /// NOT absolute address
    pub(super) suspended_stack: FramePtr,
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
            suspended_stack: FramePtr(exception_frame),
        }
    }

    fn apply<const U: usize>(&mut self, args: AbiArguments<U>) {
        let stack = self.suspended_stack.as_mut();
        stack.abi_regs[..U].copy_from_slice(args.pushed());
    }

    pub fn sp(&self) -> FramePtr {
        self.suspended_stack
    }
}

pub struct PendingTask(Task);

impl PendingTask {
    pub fn new(task: Task) -> PendingTask {
        PendingTask(task)
    }

    // TODO
    pub fn resolve_args<const U: usize>(mut self, args: AbiArguments<U>) -> Task {
        self.0.apply(args);
        self.0
    }
}
