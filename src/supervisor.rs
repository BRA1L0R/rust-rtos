use core::cell::RefCell;

use crate::scheduler::Scheduler;

extern crate alloc;

use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::SCB,
};

/// global supervisor instance
static SUPERVISOR: Mutex<RefCell<Option<Supervisor>>> = Mutex::new(RefCell::new(None));

pub struct Supervisor {
    pub(crate) sched: Scheduler,
    system_control: SCB,
}

impl Supervisor {
    pub fn new(system_control: SCB, sched: Scheduler) -> Self {
        Supervisor {
            sched,
            system_control,
        }
    }

    pub(crate) fn pend_switch(&self) {
        unsafe { self.system_control.icsr.modify(|reg| reg | 0x1 << 28) };
    }
}

pub fn init_supervisor(sup: Supervisor) {
    free(|cs| SUPERVISOR.borrow(cs).replace(Some(sup)));
}

/// Note: code executed in the closure is subject
/// to a critical section
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
