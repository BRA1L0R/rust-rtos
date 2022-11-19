#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use alloc_cortex_m::CortexMHeap;
use cortex_m_rt::entry;

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;

use stm32_transponder::{api::r#yield, supervisor::SupervisorBuilder};
use stm32f0xx_hal as _;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

extern "C" fn first_task() -> ! {
    let mut a = 10;
    loop {
        r#yield();
        a += 1;
    }
}

extern "C" fn second_task() -> ! {
    let mut a: i32 = 20;
    loop {
        r#yield();
        a += 1;
    }
}

extern "C" fn third_task() -> ! {
    let mut a: i32 = 20;
    loop {
        r#yield();
        a += 1;
    }
}

#[entry]
fn entry() -> ! {
    // let heap_start: *const u8 = unsafe { &_heap_start };
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as _, 3100) };

    let per = cortex_m::Peripherals::take().unwrap();
    let supervisor = SupervisorBuilder::new(per)
        .add_task(first_task)
        .add_task(second_task)
        .add_task(third_task);

    let free = ALLOCATOR.free();
    hprintln!("Free memory: {}", free);

    supervisor.start();
}
