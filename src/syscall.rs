use crate::{drivers, supervisor};
use core::fmt::Debug;
use core::fmt::Write;

#[repr(u32)]
pub enum SVCallId {
    Yield = 0x00,
    Print,
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
unsafe extern "C" fn svcall(id: u32, args: CallArguments) {
    let id = SVCallId::try_from(id).unwrap();
    let mut args = args.into_iter();

    match id {
        SVCallId::Yield => (),
        SVCallId::Print => {
            let data: *const u8 = next_as!(args).unwrap();
            let length = next_as!(args).unwrap();

            let data = core::slice::from_raw_parts(data, length);
            let data = core::str::from_utf8_unchecked(data);

            write!(drivers::tty_writer(), "{data}").unwrap();
        }
    }

    supervisor::with_supervisor(|spv| spv.pend_switch());
}
