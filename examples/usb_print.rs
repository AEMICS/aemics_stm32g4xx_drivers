#![no_std]
#![no_main]

//! USB Print example
//!
//! This example shows an interrupt driver USB driver set up to print "Hello World" once every second.
//! It has to be interrupt driven due to the delay used in the program's loop function.
//!
//! Currently the interrupt logic has not been wrapped into a USB driver.

use hal::panic_semihosting as _; //Panic Handler
use aemics_stm32g4xx_hal as hal;

use hal::{
    rcc::{Config, RccExt},
    stm32,
};

use aemics_drivers::usb::{UsbDriver, Write};

use aemics_stm32g4xx_hal::preludes::{
    default::*,
    digital::*,
    delay::*,
};

use aemics_stm32g4xx_hal::pwr::PwrExt;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    //Enable the HSI48 clock (48MHz) for driving the USB peripheral. This also enables the HSI clock which is used as the SYST clock.
    let pwr = dp.PWR.constrain().freeze();
    let mut rcc = dp.RCC.freeze(Config::hsi48(), pwr);

    let mut delay = cp.SYST.delay(&rcc.clocks);

    let mut usb_driver = UsbDriver::new(dp.TIM2, dp.USB, &rcc.clocks);

    let gpiob = dp.GPIOB.split(&mut rcc);
    let mut led = gpiob.pb7.into_push_pull_output();


    loop
    {
        usb_driver.write_ln(b"Hello World!").unwrap();
        //wfi();
        led.toggle().unwrap();
        delay.delay_ms(1000);
    }
}

