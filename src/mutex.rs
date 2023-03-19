use core::cell::{Ref, RefCell, RefMut};

use cortex_m::interrupt::{CriticalSection, Mutex as CMutex};

// temporary solution until critical_section is
// stabilized on cortex-m

pub struct Mutex<T>(CMutex<RefCell<T>>);

impl<T: Default> Default for Mutex<T> {
    fn default() -> Self {
        Mutex(CMutex::new(RefCell::default()))
    }
}

impl<T> Mutex<T> {
    pub const fn new(inner: T) -> Mutex<T> {
        Mutex(CMutex::new(RefCell::new(inner)))
    }

    pub fn borrow_mut<'a: 'cs, 'cs>(&'a self, cs: &'cs CriticalSection) -> RefMut<'cs, T> {
        self.0.borrow(cs).borrow_mut()
    }

    pub fn borrow<'a: 'cs, 'cs>(&'a self, cs: &'cs CriticalSection) -> Ref<'cs, T> {
        self.0.borrow(cs).borrow()
    }
}
