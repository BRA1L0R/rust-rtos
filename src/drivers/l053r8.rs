use stm32l0xx_hal::{
    gpio::GpioExt,
    pac::Peripherals,
    prelude::RateExtensions,
    // timer::,
    rcc::{Config, RccExt},
    serial::{self, Serial, USART2},
};

pub type SerialTty = Serial<USART2>;

pub fn init_drivers() {
    let peripherals = Peripherals::take().unwrap();

    let mut rcc = peripherals.RCC.freeze(Config::hsi16());
    let gpioa = peripherals.GPIOA.split(&mut rcc);

    let serial = Serial::usart2(
        peripherals.USART2,
        gpioa.pa2,
        gpioa.pa3,
        serial::Config::default().baudrate(115200u32.Bd()),
        &mut rcc,
    )
    .unwrap();

    super::TTY.populate(serial)
}
