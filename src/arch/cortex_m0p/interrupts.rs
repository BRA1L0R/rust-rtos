use core::arch::asm;

/// PendSV is THE context-switcher.
/// Saves the task, decides what to schedule next and loads it
#[export_name = "PendSV"]
#[naked]
unsafe extern "C" fn pendsv_trampoline() {
    asm!(
        "
            bl save_task
        0:
            bl context_switch // decide what task to start next
            bl load_task

            // function returns EXC_RET as second argument
            ldr r0, =0xFFFFFFFD
            bx r0
        ",
        options(noreturn)
    )
}

#[export_name = "SVCall"]
#[naked]
unsafe extern "C" fn svcall_trampoline() {
    asm!(
        "
            // Syscall ID is now passed
            // as a parameter so the next
            // commented section is not
            // used

            // mrs r0, psp
            // // load address from stack
            // ldr r0, [r0, #0x18]
            // // load byte and put it in
            // subs r0, #2
            // ldrb r0, [r0]
            
            push {{lr}}
            bl svcall
            pop {{pc}}
        ",
        options(noreturn)
    )
}
