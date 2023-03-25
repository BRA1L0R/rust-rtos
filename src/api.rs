use core::arch::asm;

use super::syscall::ids::SVCallId;

macro_rules! syscall {
    ($id:expr $(,$n:tt : $val:ident)*) => {
        asm!("SVC #0", in("r0") $id as u32, $(in($n) $val, )*)
    };

    ($id:expr $(,$n:tt : $val:ident)* => $or:tt : $oe:ident $(, $b:tt : $out:ident)*) => {
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

pub fn free() -> (usize, usize) {
    let (used, free): (usize, usize);
    unsafe { syscall!(SVCallId::FreeMem => "r0": used, "r1": free) };
    (used, free)
}
