#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

use nrf52840_hal::target::{
    Peripherals, TIMER0
};
use nrf52840_hal::gpio::{
    GpioExt, OpenDrainConfig::*, Level::*,
};
use embedded_hal::digital::v2::OutputPin;

fn delay(count: u16) {
    for _ in 0 .. count {
        asm::nop();
    }
}

fn config_timer0(timer: &TIMER0, period_usec: u32) {
    timer.mode.write(|w| w.mode().timer());
    timer.bitmode.write(|w| w.bitmode()._32bit());
    unsafe {
        timer.prescaler.write(|w| w.prescaler().bits(4)); // 16MHz / (2^4) = 1MHz clock
        timer.cc[0].write(|w| w.cc().bits(period_usec));
    }
    timer.shorts.write(|w| w.compare0_clear().enabled());
}

fn start_timer0(timer: &TIMER0) {
    timer.tasks_start.write(|w| w.tasks_start().set_bit());
}

fn wait_timer0(timer: &TIMER0) {
    let ev = &timer.events_compare[0];
    loop {
        if ev.read().events_compare().bit_is_set() {
            ev.write(|w| w.events_compare().clear_bit());
            return;
        }
    }
}

#[entry]
fn main() -> ! {
    let pers = Peripherals::take().unwrap();
    let p0 = pers.P0.split();
    let mut led = p0.p0_07.into_open_drain_output(Disconnect0Standard1, Low);
    config_timer0(&pers.TIMER0, 2_000_000);
    start_timer0(&pers.TIMER0);
    
    loop {
        led.set_high().unwrap();
        wait_timer0(&pers.TIMER0);
        led.set_low().unwrap();
        wait_timer0(&pers.TIMER0);
    }
}
