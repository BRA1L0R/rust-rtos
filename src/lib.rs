#![no_std]
#![feature(naked_functions, new_uninit, asm_const)]

// use cortex_m::register::control::Control;

use allocator::init_allocator;
use drivers::init_drivers;
use scheduler::Scheduler;
use supervisor::{init_supervisor, Supervisor};

mod allocator;
pub mod api;
mod arch;
mod drivers;
mod scheduler;
mod supervisor;
mod syscall;

pub type TaskEntrypoint = fn() -> !;

pub struct KernelBuilder(());

impl KernelBuilder {
    /// Safety for `per` makes it instantiable only one
    /// time so we are sure this function is called only
    /// one time
    pub fn new(per: cortex_m::Peripherals) -> Self {
        // init allocator
        // TODO: change with dynamic size
        unsafe { init_allocator(cortex_m_rt::heap_start(), 2048) }

        // init devices
        init_drivers();

        // init scheduler and supervisor
        let scheduler = Scheduler::new(per.SYST);
        init_supervisor(Supervisor::new(per.SCB, scheduler));

        Self(())
    }

    /// Allocates a new stack and sets the default registers
    /// for the task.
    ///
    /// ### Panic
    /// Panics if there isn't enough space for a new stack
    /// or for an additional task on the main stack
    pub fn add_task(self, entry: TaskEntrypoint) -> Self {
        use scheduler::task::Task;

        supervisor::with_supervisor(|spv| spv.sched.add_task(Task::create(entry)));

        self
    }

    /// Starts the supervisor and enters a supervised context
    ///
    /// # Panic
    /// Panics if called from an unprivileged context or if
    /// the current stack used is not MSP (main)
    pub fn start(self) -> ! {
        use crate::arch::switching::start_cold;
        use cortex_m::register::control::{self, Npriv, Spsel};

        let control = control::read();

        assert_eq!(
            control.npriv(),
            Npriv::Privileged,
            "starting a supervised context in unprivileged mode"
        );
        assert_eq!(
            control.spsel(),
            Spsel::Msp,
            "starting a supervised context using the process stack pointer"
        );

        let stack_pointer = supervisor::with_supervisor(|spv| {
            spv.sched.start_systick();
            spv.sched.current().sp()
        });

        // safety: it is the right time to call
        unsafe { start_cold(stack_pointer) }
    }
}

// unsafe fn modify_control(m: impl Fn(&mut Control)) {
//     use cortex_m::register::control;

//     let mut control_reg = control::read();
//     m(&mut control_reg);
//     control::write(control_reg);
// }
