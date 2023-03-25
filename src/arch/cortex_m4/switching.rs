use core::arch::asm;

use crate::scheduler::task::{ExtendedFrame, FramePtr};

#[no_mangle]
#[naked]
unsafe extern "C" fn save_task() {
    asm!(
        "
            mrs r0, psp
            mrs r3, control
            stmdb r0, {{r3-r12}}

            bx lr
        ",
        options(noreturn)
    )
}

/// ### Safety
/// psp must be aligned to a standard
/// hardware exception frame, and the task
/// must have exited and saved its ExtendedFrame
/// on its stack
pub unsafe fn current_extended() -> FramePtr {
    let extended: *mut ExtendedFrame;

    asm!(
        "
            mrs {0}, psp
            subs {0}, #40
        ", 
        out(reg) extended
    );

    FramePtr::new(extended)
}

#[no_mangle]
#[naked]
unsafe extern "C" fn load_task(sp: FramePtr) {
    asm!(
        "
            ldmia r0!, {{r3-r12}}
            msr psp, r0
            msr control, r3

            isb // sync new sp

            bx lr
        ",
        options(noreturn)
    )
}

#[naked]
pub(crate) unsafe extern "C" fn start_cold(sp: FramePtr) -> ! {
    asm!(
        "
            bl load_task

            mov r0, sp          // load_task changes sp to psp
            ldr r0, [r0, #0x18] // offset to entry point
            bx r0
        ",
        options(noreturn)
    )
}
