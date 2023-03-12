use core::cell::RefCell;
use cortex_m::interrupt::{free, Mutex};

#[cfg(feature = "l053r8")]
pub mod l053r8;
#[cfg(feature = "l053r8")]
pub use l053r8::*;

static TTY: Tty = Tty::empty();

pub fn tty_writer() -> impl core::fmt::Write {
    struct TtyWriter;

    impl core::fmt::Write for TtyWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            free(|cs| {
                TTY.serial
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .write_str(s)
            })
        }
    }

    TtyWriter
}

pub struct Tty {
    serial: Mutex<RefCell<Option<SerialTty>>>,
}

impl Tty {
    pub const fn empty() -> Self {
        let serial = Mutex::new(RefCell::new(None));
        Self { serial }
    }

    pub fn populate(&self, serial: SerialTty) {
        free(|cs| self.serial.borrow(cs).replace(Some(serial)));
    }
}
