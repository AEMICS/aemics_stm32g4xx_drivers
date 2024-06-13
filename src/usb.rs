use aemics_stm32g4xx_hal as hal;


use hal::{
    //rcc::{Config, RccExt},
    stm32,
};

use hal::preludes::{
    default::*,
    interrupts::*,
    timers::*,
    usb::*
};


use hal::stm32g4::stm32g473::{TIM2, USB};
use hal::rcc::Clocks;

static MUTEX_TIM2: Mutex<RefCell<Option<CountDownTimer<stm32::TIM2>>>> =
    Mutex::new(RefCell::new(None));

static mut MUTEX_USB_SERIAL: Mutex<RefCell<Option<SerialPort<UsbBusType>>>> = Mutex::new(RefCell::new(None));

static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;

static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

pub struct UsbDriver {
}

pub trait Write {
    fn write(&mut self, data: &[u8]) -> Result<usize, UsbError>;
    fn write_ln(&mut self, data: &[u8]) -> Result<usize, UsbError>;
}

pub trait Read {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, UsbError>;
}

impl UsbDriver {
    pub fn new(timer: TIM2, usb: USB, clocks: &Clocks) -> UsbDriver {

        let usb = USBObj { usb };

        unsafe {
            let bus = UsbBus::new(usb);

            USB_BUS = Some(bus);

            cortex_m::interrupt::free(|cs| MUTEX_USB_SERIAL.borrow(cs).borrow_mut().replace(SerialPort::new(USB_BUS.as_ref().unwrap())));

            let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
                .manufacturer("AEMICS")
                .product("Serial Port")
                .device_class(USB_CLASS_CDC)
                .build();

            USB_DEVICE = Some(usb_dev);
        }

        let timer2 = Timer::new(timer, clocks);

        let mut timer2_i = timer2.start_count_down_us(10.micros());
        timer2_i.listen(Event::TimeOut);

        cortex_m::interrupt::free(|cs| MUTEX_TIM2.borrow(cs).borrow_mut().replace(timer2_i));

        unsafe {
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
        }

        UsbDriver{}
    }
}

impl Write for UsbDriver {
    fn write(&mut self, data: &[u8]) -> Result<usize, UsbError> {
        cortex_m::interrupt::free(|cs| {
            if let Some(ref mut serial) = unsafe { MUTEX_USB_SERIAL.borrow(cs).borrow_mut().deref_mut() } {
                match serial.write(data) {
                    Ok(count) => match serial.flush() {
                        Ok(_) | Err(UsbError::WouldBlock) => Ok(count),
                        Err(err) => Err(err),
                    },
                    Err(e) => Err(e),
                }
            } else {
                Err(UsbError::WouldBlock)
            }
        })
    }

    fn write_ln(&mut self, data: &[u8]) -> Result<usize, UsbError>  {
        // First write the data
        let count = self.write(data)?;

        // Then write the newline character(s)
        // Here we assume a newline is "\r\n" as commonly used in serial communication
        let newline = b"\r\n";
        self.write(newline).map(|_| count)
    }
}

impl Read for UsbDriver {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, UsbError> {
        cortex_m::interrupt::free(|cs| {
            if let Some(ref mut serial) = unsafe { MUTEX_USB_SERIAL.borrow(cs).borrow_mut().deref_mut() } {
                match serial.read(buffer) {
                    Ok(count) => match serial.flush() {
                        Ok(_) | Err(UsbError::WouldBlock) => Ok(count),
                        Err(err) => Err(err),
                    },
                    Err(e) => Err(e),
                }
            } else {
                Err(UsbError::WouldBlock)
            }
        })
    }
}

#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut t2) = MUTEX_TIM2.borrow(cs).borrow_mut().deref_mut()
        {
            usb_interrupt();
            t2.clear_interrupt(Event::TimeOut);
        }
    });
}


fn usb_interrupt() {
    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    //let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut serial) = unsafe { MUTEX_USB_SERIAL.borrow(cs).borrow_mut().deref_mut() }
        {
            if !usb_dev.poll(&mut [serial]) {
                return;
            }
        }
    });


}

/*
#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut t2) = MUTEX_TIM2.borrow(cs).borrow_mut().deref_mut()
        {
            usb_interrupt();
            t2.clear_interrupt(Event::TimeOut);
        }
    });
}

fn usb_interrupt() {
    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    //let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut serial) = unsafe { MUTEX_USB_SERIAL.borrow(cs).borrow_mut().deref_mut() }
        {
            if !usb_dev.poll(&mut [serial]) {
                return;
            }
        }
    });


}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    //Enable the HSI48 clock (48MHz) for driving the USB peripheral. This also enables the HSI clock which is used as the SYST clock.
    let pwr = dp.PWR.constrain().freeze();
    let mut rcc = dp.RCC.freeze(Config::hsi48(), pwr);

    let mut delay = cp.SYST.delay(&rcc.clocks);

    let usb = USBObj { usb: dp.USB };

    unsafe {
        let bus = UsbBus::new(usb);

        USB_BUS = Some(bus);

        cortex_m::interrupt::free(|cs| MUTEX_USB_SERIAL.borrow(cs).borrow_mut().replace(SerialPort::new(USB_BUS.as_ref().unwrap())));

        let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("AEMICS")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build();

        USB_DEVICE = Some(usb_dev);
    }


    let gpiob = dp.GPIOB.split(&mut rcc);
    let mut led = gpiob.pb7.into_push_pull_output();

    let timer2 = Timer::new(dp.TIM2, &rcc.clocks);

    let mut timer2_i = timer2.start_count_down_us(10.micros());
    timer2_i.listen(Event::TimeOut);

    cortex_m::interrupt::free(|cs| MUTEX_TIM2.borrow(cs).borrow_mut().replace(timer2_i));

    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    loop
    {
        cortex_m::interrupt::free(|cs| {
            if let Some(ref mut serial) = unsafe { MUTEX_USB_SERIAL.borrow(cs).borrow_mut().deref_mut() }
            {
                serial.write(b"Hello World!\n").unwrap();
            }
        });

        //wfi();
        led.toggle().unwrap();
        delay.delay_ms(1000);
    }
}

*/