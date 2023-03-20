#![no_std]
#![no_main]

extern crate alloc;

use core::{hint::black_box, sync::atomic};

use cortex_m_rt::entry;
use panic_semihosting as _;

use rust_rtos::{
    api::{self, print},
    KernelBuilder,
};

struct Reader;
impl Iterator for Reader {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        Some(api::read_char())
    }
}

fn first_task() -> ! {
    loop {
        let line: heapless::String<10> =
            Reader.take_while(|&c| c != b'\r').map(char::from).collect();

        print(&line);
        print("\r\n");
    }
}

fn second_task() -> ! {
    loop {
        // busy loop
        (0..100_000).map(black_box).for_each(drop);
        // api::free();
        api::print("reached 100'000!\r\n");
    }
}

fn third_task() -> ! {
    let mut _a: i32 = 20;
    loop {
        // print("Hello world from task 3\r\n");
        // r#yield();
        _a += 1;
    }
}

#[entry]
fn entry() -> ! {
    let per = cortex_m::Peripherals::take().unwrap();
    let kernel = KernelBuilder::new(per)
        .add_task(first_task)
        .add_task(second_task);
    // .add_task(third_task);

    // safety: not in a critical context
    unsafe { kernel.init_drivers() }.start();
}
