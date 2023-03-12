use core::cell::RefCell;

use crate::scheduler::Scheduler;

extern crate alloc;

use cortex_m::interrupt::{free, Mutex};

/// global supervisor instance
static SUPERVISOR: Mutex<RefCell<Option<Supervisor>>> = Mutex::new(RefCell::new(None));

pub struct Supervisor {
    pub(crate) sched: Scheduler,
    per: cortex_m::Peripherals,
}

impl Supervisor {
    pub fn new(per: cortex_m::Peripherals) -> Self {
        Supervisor {
            sched: Default::default(),
            per,
        }
    }

    pub(crate) fn pend_switch(&self) {
        unsafe { self.per.SCB.icsr.modify(|reg| reg | 0x1 << 28) };
    }
}

pub fn init_supervisor(sup: Supervisor) {
    free(|cs| SUPERVISOR.borrow(cs).replace(Some(sup)));
}

/// Safety:
/// This is a critical section. All interrupts are disabled. Pending
/// interrupts will get handled as soon as the closure is over.
///
/// Panic:
/// panics if not executed inside of a supervised context.
pub(crate) fn with_supervisor<T>(m: impl Fn(&mut Supervisor) -> T) -> T {
    free(|cs| {
        let mut supervisor = SUPERVISOR.borrow(cs).borrow_mut();
        let supervisor = supervisor
            .as_mut()
            .expect("not running inside a supervised context");

        m(supervisor)
    })
}
