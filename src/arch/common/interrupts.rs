use core::arch::asm;

/// PendSV is THE context-switcher.
/// Saves the task, decides what to schedule next and loads it
#[export_name = "PendSV"]
#[naked]
unsafe extern "C" fn pendsv_trampoline() {
    asm!(
        "
            bl context_switch // decide what task to start next
            bl load_task

            // function returns EXC_RET as second argument
            // TODO: change this to dynamic
            ldr r0, =0xFFFFFFFD
            bx r0
        ",
        options(noreturn)
    )
}

#[export_name = "SysTick"]
#[naked]
unsafe extern "C" fn systick_trampoline() {
    asm!(
        "            
            push {{lr}} // push before anything or else other bl-s might interfere

            bl save_task
            bl system_tick

            pop {{pc}} // pop to pc triggers exit
        ",
        options(noreturn)
    )
}

#[export_name = "SVCall"]
#[naked]
unsafe extern "C" fn svcall_trampoline() {
    asm!(
        "
            push {{lr}}

            push {{r0-r3}} // preserve system-call call arguments
            bl save_task
            pop {{r0-r3}}

            bl svcall

            pop {{pc}}
        ",
        options(noreturn)
    )
}
