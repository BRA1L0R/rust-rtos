use core::arch::asm;

use crate::scheduler::task::ExtendedFrame;

#[export_name = "PendSV"]
#[naked]
unsafe extern "C" fn pendsv_trampoline() {
    asm!(
        "
            bl save_task
        0:
            bl pendsv
            bl load_task

            // function returns EXC_RET as second argument
            ldr r0, =0xFFFFFFFD
            bx r0
        ",
        options(noreturn)
    )
}

#[no_mangle]
#[naked]
unsafe extern "C" fn save_task() -> *mut ExtendedFrame {
    asm!(
        "
            mrs r0, psp
            subs r0, #40        // create space for *ExtendedFrame
            mov r1, r0          // keep r0 for return

            mrs r3, control         // load control
            stm r1!, {{r3,r4-r7}}   // store control,r4-r7

            mov r3, r8          // shift registers
            mov r4, r9
            mov r5, r10
            mov r6, r11
            mov r7, r12
            stm r1!, {{r3-r7}}  // store r8-r12

            bx lr
        ",
        options(noreturn)
    )
}

#[no_mangle]
#[naked]
unsafe extern "C" fn load_task(stack_pointer: *mut ExtendedFrame) {
    asm!(
        "
            adds r0, #20        // writeback constraint workaround
            ldm r0!, {{r3-r7}}  // load control, r8-r12
            mov r8, r3          // shit registers
            mov r9, r4
            mov r10, r5
            mov r11, r6
            mov r12, r7

            // load new stack pointer which is now aligned
            // because ldm is a writeback instruction
            msr psp, r0

            subs r0, #40
            ldm r0!, {{r3,r4-r7}}      // load control,r4-r7
            msr control, r3

            isb // sync new sp

            bx lr
        ",
        options(noreturn)
    )
}

#[naked]
pub(crate) extern "C" fn start_cold(stack_pointer: *mut ExtendedFrame) -> ! {
    unsafe {
        asm!(
            "
                bl load_task

                mov r0, sp          // load_task changes sp to psp
                ldr r0, [r0, #0x18]
                bx r0
            ",
            options(noreturn)
        )
    }
}

#[export_name = "SVCall"]
#[naked]
unsafe extern "C" fn svcall_trampoline() {
    asm!(
        "
            mrs r0, psp

            // load address from stack
            ldr r0, [r0, #0x18]
            // load byte and put it in
            subs r0, #2
            ldrb r0, [r0]
            
            push {{lr}}
            bl svcall
            pop {{pc}}
        ",
        options(noreturn)
    )
}
