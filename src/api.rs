use core::arch::asm;

use super::syscall::SVCallId;

#[inline(always)]
pub fn r#yield() {
    unsafe { asm!("SVC #0") }
}
