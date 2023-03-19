#![no_std]
#![no_main]

pub mod mutex;

extern crate alloc;

use cortex_m_rt::entry;
use panic_semihosting as _;

use rust_rtos::{
    api::{print, read_char},
    KernelBuilder,
};

fn first_task() -> ! {
    let mut _a = 10;
    loop {
        // print("Task 1\r\n");
        let _char = read_char();
        _a += 1;
    }
}

fn second_task() -> ! {
    let mut _a: i32 = 20;
    loop {
        // print("Task 2\r\n");
        _a += 1;
    }
}

fn third_task() -> ! {
    let mut _a: i32 = 20;
    loop {
        // print("Hello world from task 3\r\n");
        _a += 1;
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
