use cortex_m::register::control::{self};

use crate::supervisor;
use core::fmt::Debug;

#[repr(u8)]
pub enum SVCallId {
    Yield = 0x0,
}

pub struct InvalidSVC;
impl Debug for InvalidSVC {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "invalid SVC identifier")
    }
}

impl TryFrom<u8> for SVCallId {
    type Error = InvalidSVC;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(SVCallId::Yield),
            _ => Err(InvalidSVC),
        }
    }
}

#[no_mangle]
extern "C" fn svcall(id: u8) {
    let id = SVCallId::try_from(id).unwrap();
    let privilege = control::read().npriv();

    match id {
        SVCallId::Yield => (),

        _ => panic!("unknown syscall id or unsufficient privileges"),
    }

    supervisor::with_supervisor(|spv| spv.pend_sv());
}
