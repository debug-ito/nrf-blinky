#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

use nrf52840_pac::Peripherals;
use nrf52840_hal::gpio::{GpioExt, OpenDrainConfig::Disconnect0Standard1, Level::Low};
use embedded_hal::digital::v2::OutputPin;

fn delay(count: u16) {
    for _ in 0 .. count {
        asm::nop();
    }
}

#[entry]
fn main() -> ! {
    let pers = Peripherals::take().unwrap();
    let p0 = pers.P0.split();
    let mut led = p0.p0_07.into_open_drain_output(Disconnect0Standard1, Low);
    const DELAY : u16 = 20000;
    
    loop {
        led.set_high().unwrap();
        delay(3 * DELAY);
        led.set_low().unwrap();
        delay(DELAY);
    }
}
