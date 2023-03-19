#[cfg(feature = "l053r8")]
pub mod l053r8;
use core::cell::RefMut;

use cortex_m::{interrupt::CriticalSection, prelude::_embedded_hal_serial_Read};
#[cfg(feature = "l053r8")]
pub use l053r8::*;

use crate::{mutex::Mutex, scheduler::task::PendingTask, supervisor, toinit::ToInit};

// Safety: must be called OUTSIDE a critical context
pub unsafe fn init_drivers() {
    init_spec();
}

static SERIAL: Mutex<ToInit<Serial>> = Mutex::new(ToInit::uninit());

pub struct Serial {
    inner: SerialSpec,

    buffer: heapless::Deque<u8, 10>,
    subscribed: Option<PendingTask>,
}

impl Serial {
    pub fn new(inner: SerialSpec) -> Self {
        Self {
            inner,
            buffer: heapless::Deque::new(),
            subscribed: None,
        }
    }

    pub fn read_char(&mut self) -> Option<u8> {
        if let buffered @ Some(_) = self.buffer.pop_front() {
            return buffered;
        };

        self.inner.read().ok()
    }

    pub fn subscribe(&mut self, task: PendingTask) {
        self.subscribed.replace(task);
    }
}

/// generic implementation which is
/// called by the specific handlers
///
/// Safety: must be called from an interrupt that
/// cannot be preempted by any other interrupts
unsafe fn interrupt_handler() {
    let cs = CriticalSection::new();

    let mut serial = serial(&cs);

    // If interrupt was called then we are
    // sure to have data
    let data = serial.inner.read().unwrap();

    match serial.subscribed.take() {
        None => serial.buffer.push_back(data).unwrap(),
        Some(task) => {
            let task = task.resolve_args(data as u32);
            supervisor::supervisor(&cs).sched.schedule_task(task);
        }
    };
}

/// Panic: if serial is not initialized
pub fn serial(cs: &CriticalSection) -> RefMut<'_, Serial> {
    RefMut::map(SERIAL.borrow_mut(cs), |f| f.unwrap_mut())
}
