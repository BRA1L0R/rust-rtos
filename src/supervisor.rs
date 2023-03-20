use core::cell::RefMut;

use crate::{mutex::Mutex, scheduler::Scheduler, toinit::ToInit};
use cortex_m::{
    interrupt::{free, CriticalSection},
    peripheral::SCB,
};

extern crate alloc;

/// global supervisor instance
static SUPERVISOR: Mutex<ToInit<Supervisor>> = Mutex::new(ToInit::uninit());

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
        // Safety: setting the flag for pendsv is not
        // actually unsafe
        unsafe { self.system_control.icsr.modify(|reg| reg | 0x1 << 28) };
    }
}

pub fn init_supervisor(sup: Supervisor) {
    free(|cs| SUPERVISOR.borrow_mut(cs).init(sup));
}

pub(crate) fn supervisor(cs: &CriticalSection) -> RefMut<'_, Supervisor> {
    RefMut::map(SUPERVISOR.borrow_mut(cs), |f| {
        f.expect_mut("not running in a supervised context")
    })
}
