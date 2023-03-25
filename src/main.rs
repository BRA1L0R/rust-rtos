#![no_std]
#![no_main]

extern crate alloc;

use core::hint::black_box;

use cortex_m_rt::entry;
use panic_semihosting as _;
use stm32f4xx_hal as _;

use rust_rtos::{
    api::{self, free, r#yield},
    KernelBuilder,
};

fn first_task() -> ! {
    loop {
        // let line: heapless::String<10> =
        //     Reader.take_while(|&c| c != b'\r').map(char::from).collect();

        // print(&line);
        // print("\r\n");
        api::r#yield();
    }
}

fn second_task() -> ! {
    loop {
        // busy loop
        (0..100_000).map(black_box).for_each(drop);
        // api::print("reached 100'000!\r\n");
        api::r#yield();
    }
}

fn third_task() -> ! {
    loop {
        let (used, free) = free();
        r#yield();
    }
}

#[entry]
fn entry() -> ! {
    let per = cortex_m::Peripherals::take().unwrap();
    let kernel = KernelBuilder::new(per)
        .add_task(first_task)
        .add_task(second_task)
        .add_task(third_task);

    // safety: not in a critical context
    unsafe { kernel.init_drivers() }.start();
}
