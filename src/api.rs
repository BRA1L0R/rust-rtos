use core::arch::asm;

use super::syscall::SVCallId;

// unsafe extern "C" fn syscall(_id: u32, _args: ...) {
//     asm!("SVC #0", in("r0") 10);
// }

macro_rules! syscall {
    ($id:expr $(,$n:tt : $val:expr)*) => {
        asm!("SVC #0", in("r0") $id as u32, $(in($n) $val, )*)
    };

    ($id:expr $(,$n:tt : $val:expr)* => $or:tt : $oe:tt $(, $b:tt : $out:tt)*) => {
        asm!("SVC #0", in("r0") $id as u32, $(in($n) $val, )* lateout($or) $oe, $(lateout($b) $out, )*)
    }
}

pub fn r#yield() {
    // unsafe { syscall(SVCallId::Yield as _) }
    unsafe { syscall!(SVCallId::Yield) }
}

pub fn print(data: &str) {
    let dataptr = data.as_ptr();
    let length = data.len();

    unsafe { syscall!(SVCallId::Print, "r1": dataptr, "r2": length) }
}

pub fn read_char() -> u8 {
    let res: u8;
    unsafe { syscall!(SVCallId::ReadChar => "r0": res) };

    res
}
