#![no_std]
#![no_main]

use aemics_stm32g4xx_hal as hal;
use aemics_stm32g4xx_hal::delay::SYSTDelayExt;

use hal::preludes::{
    default::*,
};
use aemics_drivers::usb::{UsbDriver, Write, Read};


#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    //Enable the HSI48 clock (48MHz) for driving the USB peripheral. This also enables the HSI clock which is used as the SYST clock.
    let pwr = dp.PWR.constrain().freeze();
    let mut rcc = dp.RCC.freeze(Config::hsi48(), pwr);

    let mut usb_driver = UsbDriver::new(dp.TIM2, dp.USB, &rcc.clocks);

    loop {

        let mut buf = [0u8; 64];

        //Read incoming data
        match usb_driver.read(&mut buf) {
            Ok(_) => {
                //Swap lower case characters for upper case.
                for c in buf[0..64].iter_mut() {
                    if b'a' <= *c && *c <= b'z' {
                        *c &= !0x20;
                    }
                }

                //Send back capital letter response.
                usb_driver.write(&buf).unwrap();
            }
            _ => {}
        }
    }
}
