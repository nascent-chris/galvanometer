#![deny(unsafe_code)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

// use cortex_m_semihosting as _;
// use cortex_m_semihosting::hprintl;
// use panic_halt as _;
// use panic_semihosting as _;

use panic_semihosting as _;

use cortex_m_semihosting::{hprint, hprintln};
use stm32f1xx_hal::{
    self as _,
    usb::{Peripheral, UsbBus},
};

use cortex_m::asm;
use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    time::ms,
    timer::{Channel, Tim2NoRemap},
};
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let rcc = p.RCC.constrain();

    // let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    let mut afio = p.AFIO.constrain();

    let mut gpioa = p.GPIOA.split();
    let mut gpioc = p.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // TIM2
    let c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    let c2 = gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl);
    let c3 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let pins = (c1, c2, c3);

    let mut pwm = p
        .TIM2
        .pwm_hz::<Tim2NoRemap, _, _>(pins, &mut afio.mapr, 1.kHz(), &clocks);

    // Enable clock on each of the channels
    pwm.enable(Channel::C1);
    pwm.enable(Channel::C2);
    pwm.enable(Channel::C3);

    pwm.set_period(10.kHz());

    let max_pwm = pwm.get_max_duty();
    hprint!("Max PWM: {}", max_pwm);

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    cortex_m::asm::delay(clocks.sysclk().raw() / 100);

    let usb = Peripheral {
        usb: p.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    };
    let usb_bus = UsbBus::new(usb);

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();

    let duty_cycle = duty_cycle_from_desired_gauge_reading(150.0) as u16;
    pwm.set_duty(Channel::C3, duty_cycle);
    pwm.set_duty(Channel::C2, duty_cycle);

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 64];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                // decode first byte of buffer as a percentage
                let desired_percentage = (buf[0] as f64 / 255.0) * 100.0;
                hprintln!("Desired percentage: {}", desired_percentage);
                let duty_cycle =
                    duty_cycle_from_desired_gauge_reading(desired_percentage * 3.0) as u16;
                pwm.set_duty(Channel::C3, duty_cycle);
                pwm.set_duty(Channel::C2, duty_cycle);

                led.set_low(); // Turn on

                // Echo back in upper case
                // for c in buf[0..count].iter_mut() {
                //     if 0x61 <= *c && *c <= 0x7a {
                //         *c &= !0x20;
                //     }
                // }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        led.set_high(); // Turn off

        /*

        let mut desired_percentage = 0.0;
        let mut ramping_up = true;
        loop {
            // let duty_cycle = duty_cycle_from_percentage(desired_percentage);
            let duty_cycle = duty_cycle_from_desired_gauge_reading(desired_percentage * 3.0) as u16;
            pwm.set_duty(Channel::C3, duty_cycle);
            pwm.set_duty(Channel::C2, duty_cycle);

            let hysteresis = 0.03;

            let should_pause = [0.0, 33.3, 50.0, 66.7, 100.0]
                .iter()
                .any(|&x| desired_percentage <= x + hysteresis && desired_percentage >= x - hysteresis);

            if should_pause {
                // hprint!("{}", desired_percentage * 3.0);
                // hprintln!("\t{}", duty_cycle);
                cortex_m::asm::delay(8_000_000);
            }

            if desired_percentage <= 0.0 && ramping_up == false {
                ramping_up = true;
            }
            if ramping_up {
                desired_percentage += 0.1;
            } else {
                desired_percentage -= 0.1;
            }

            if desired_percentage <= 100.0 + hysteresis && desired_percentage >= 100.0 - hysteresis {
                // desired_percentage = 0.0;
                ramping_up = if ramping_up { false } else { true };
            }

            cortex_m::asm::delay(20_000);
            // */
    }
}

/*
0 - 300 on gauge
desired percentage | expected gauge reading | actual gauge reading | duty cycle
-------------------|------------------------|--------------------- | ----------
0                  | 0.0                    | 0.0                  |  36
25                 | 75                     | 60                   |  68
50                 | 150                    | 130                  |  99
75                 | 225                    | 210                  |  130
100                | 300                    | 300                  |  161

*/

// y = -0.00036x^2 + 0.52171x + 35.98441
fn duty_cycle_from_desired_gauge_reading(actual_gauge_reading: f64) -> f64 {
    let a = -0.00036;
    let b = 0.52171;
    let c = 35.98441;

    ((a * actual_gauge_reading * actual_gauge_reading) + (b * actual_gauge_reading) + c) * 6.0
}
