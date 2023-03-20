use cortex_m::interrupt::CriticalSection;

use crate::{allocator, arch::switching, drivers, supervisor};
use core::fmt::Debug;

#[repr(u32)]
pub enum SVCallId {
    Yield = 0x00,
    Print,
    ReadChar,
    FreeMem,
}

pub struct InvalidSVC;
impl Debug for InvalidSVC {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "invalid SVC identifier")
    }
}

impl TryFrom<u32> for SVCallId {
    type Error = InvalidSVC;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(SVCallId::Yield),
            0x01 => Ok(SVCallId::Print),
            0x02 => Ok(SVCallId::ReadChar),
            0x03 => Ok(SVCallId::FreeMem),
            _ => Err(InvalidSVC),
        }
    }
}

#[repr(C)]
struct CallArguments {
    regs: [u32; 3],
}

impl IntoIterator for CallArguments {
    type Item = u32;
    type IntoIter = <[u32; 3] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.regs.into_iter()
    }
}

macro_rules! next_as {
    ($args:expr) => {{
        $args.next().map(|arg| arg as _)
    }};
}

/// Safety: not specifying the correct amount of arguments
/// during a syscall can cause undefined behaviour as the receiver
/// function uses non-initialized data anyways
#[no_mangle]
extern "C" fn svcall(id: u32, args: CallArguments) {
    // Safety: since sycalls have the same priority
    // as all other interrupts there's no way it can
    // get preempted
    let cs = unsafe { CriticalSection::new() };
    let mut supervisor = supervisor::supervisor(&cs);

    // save current extended frame in case
    // svcalls does not return to caller
    // Safety: comes from a trampoline that saves the
    // frame information
    let frame = unsafe { switching::current_extended() };
    supervisor.sched.save_current(frame);

    let id = SVCallId::try_from(id).unwrap();
    let mut args = args.into_iter();

    match id {
        SVCallId::Yield => supervisor.pend_switch(),
        SVCallId::Print => {
            let data: *const u8 = next_as!(args).unwrap();
            let length = next_as!(args).unwrap();

            // NOT SAFE: user could leak other task's or
            // even kernel memory
            let data = unsafe { core::slice::from_raw_parts(data, length) };
            let data = core::str::from_utf8(data).expect("utf8 encoded string");

            let mut serial = drivers::serial(&cs);
            serial.write_str(data)
        }
        SVCallId::ReadChar => {
            let mut serial = drivers::serial(&cs);
            assert!(serial.read_char().is_none());

            let task = supervisor.sched.pend_current();

            serial.subscribe(task);
            supervisor.pend_switch()
        }
        SVCallId::FreeMem => {
            let (free, used) = allocator::free();
            cortex_m::asm::bkpt();
        }
    }
}
