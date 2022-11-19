use core::cell::RefCell;

use crate::{
    arch::start_cold,
    scheduler::{task::Task, Scheduler},
    TaskEntrypoint,
};

extern crate alloc;

use cortex_m::{
    interrupt::Mutex,
    register::control::{self, Npriv, Spsel},
};

/// global supervisor instance
static SUPERVISOR: Mutex<RefCell<Option<Supervisor>>> = Mutex::new(RefCell::new(None));

pub struct Supervisor {
    pub(crate) sched: Scheduler,
    per: cortex_m::Peripherals,
}

impl Supervisor {
    fn new(per: cortex_m::Peripherals) -> Self {
        Supervisor {
            sched: Default::default(),
            per,
        }
    }

    pub(crate) fn pend_sv(&self) {
        unsafe { self.per.SCB.icsr.modify(|reg| reg | 0x1 << 28) };
    }
}

pub struct SupervisorBuilder(());

impl SupervisorBuilder {
    /// Safety for per makes it instantiable only one
    /// time so we are sure this function is called only
    /// one time
    pub fn new(per: cortex_m::Peripherals) -> Self {
        cortex_m::interrupt::free(|cs| {
            let supervisor = Supervisor::new(per);
            SUPERVISOR.borrow(cs).replace(Some(supervisor));
        });

        Self(())
    }

    /// Allocates a new stack and sets the default registers
    /// for the task.
    ///
    /// ### Panic
    /// Panics if there isn't enough space for a new stack
    /// or for an additional task on the main stack
    pub fn add_task(self, entry: TaskEntrypoint) -> Self {
        with_supervisor(|spv| spv.sched.add_task(Task::create(entry)));
        self
    }

    /// Starts the supervisor and enters a supervised context
    ///
    /// ### Panic
    /// Panics if called from an unprivileged context or if
    /// the current stack used is not MSP (main)
    pub fn start(self) -> ! {
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

        let stack_pointer = with_supervisor(|spv| spv.sched.current().sp());
        start_cold(stack_pointer);

        // panic!("exception returned in main")
    }
}

/// Safety:
/// This is a critical section. All interrupts are disabled. Pending
/// interrupts will get handled as soon as the closure is over.
///
/// Panic:
/// panics if not executed inside of a supervised context.
pub(crate) fn with_supervisor<T>(m: impl Fn(&mut Supervisor) -> T) -> T {
    use cortex_m::interrupt::free;

    free(|cs| {
        let mut supervisor = SUPERVISOR.borrow(cs).borrow_mut();
        let supervisor = supervisor
            .as_mut()
            .expect("not running inside a supervised context");

        m(supervisor)
    })
}
