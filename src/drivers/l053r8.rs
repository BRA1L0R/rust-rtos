use cortex_m::{interrupt::free, peripheral::NVIC};
use stm32l0xx_hal::{
    gpio::GpioExt,
    pac::Peripherals,
    pac::{interrupt, Interrupt},
    prelude::RateExtensions,
    rcc::{Config, RccExt},
    serial::{self, Serial, USART2},
};

// use super::interrupt_handler;

pub type SerialSpec = Serial<USART2>;

/// Safety: see caller function contract
pub(super) unsafe fn init_spec() {
    let peripherals = Peripherals::take().unwrap();

    let mut rcc = peripherals.RCC.freeze(Config::hsi16());
    let gpioa = peripherals.GPIOA.split(&mut rcc);

    let mut serial = Serial::usart2(
        peripherals.USART2,
        gpioa.pa2,
        gpioa.pa3,
        serial::Config::default().baudrate(115200u32.Bd()),
        &mut rcc,
    )
    .unwrap();

    serial.listen(serial::Event::Rxne);
    NVIC::unmask(Interrupt::USART2);

    let serial = super::Serial::new(serial);
    free(|cs| super::SERIAL.borrow_mut(cs).init(serial));
}

#[interrupt]
fn USART2() {
    // Safety: USART2 has the same priority level as all other interrupts
    unsafe { super::interrupt_handler() }
}
