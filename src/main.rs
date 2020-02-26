#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

extern crate nrf52840_hal;

use core::cell::RefCell;
use core::sync::atomic::{AtomicU8, Ordering::Relaxed};
use cortex_m::Peripherals as CorePeripherals;
use cortex_m::register::primask;
use cortex_m::asm;
use cortex_m::interrupt::{self as cm_interrupt, Mutex};
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;

// use nrf52840_pac::interrupt; // for [interrupt] attribute?

use nrf52840_hal::target::{
    interrupt, Peripherals, TIMER0 as TIMER0_t
};
use nrf52840_hal::gpio::{
    GpioExt, OpenDrainConfig::*, Level::*,
    p0::P0_07, Output, OpenDrain
};
use embedded_hal::digital::v2::OutputPin;


static EV_TIMER0: Mutex<RefCell<Option<TIMER0_t>>> = Mutex::new(RefCell::new(None));
static COUNTER: AtomicU8 = AtomicU8::new(0);
// static LED_PIN: Mutex<RefCell<Option<P0_07<Output<OpenDrain>>>>> = Mutex::new(RefCell::new(None));

fn delay(count: u16) {
    for _ in 0 .. count {
        asm::nop();
    }
}

fn assert_blink<T>(assertion: bool, led: &mut T) -> !
    where T: OutputPin,
          T::Error: core::fmt::Debug
{
    const DELAY: u16 = 20000;

    loop {
        led.set_high().unwrap();
        delay(DELAY);
        if !assertion {
            led.set_low().unwrap();
        }
        delay(DELAY * 3);
    }
}

fn config_timer0(timer: &TIMER0_t, nvic: &mut NVIC, period_usec: u32) {
    timer.mode.write(|w| w.mode().timer());
    timer.bitmode.write(|w| w.bitmode()._32bit());
    unsafe {
        timer.prescaler.write(|w| w.prescaler().bits(4)); // 16MHz / (2^4) = 1MHz clock
        timer.cc[0].write(|w| w.cc().bits(period_usec));
    }
    timer.shorts.write(|w| w.compare0_clear().enabled());
    timer.intenset.write(|w| w.compare0().set_bit());
    timer.events_compare[0].write(|w| w.events_compare().clear_bit());

    const INT: nrf52840_hal::target::Interrupt = nrf52840_hal::target::Interrupt::TIMER0;
    // nvic.enable(nrf52840_hal::target::Interrupt::TIMER0);
    unsafe {
        nvic.set_priority(INT, 1);
        NVIC::unmask(INT);
    }
    NVIC::unpend(INT);
}

fn start_timer0(timer: &TIMER0_t) {
    timer.tasks_start.write(|w| w.tasks_start().set_bit());
}

fn wait_timer0(timer: &TIMER0_t) {
    let ev = &timer.events_compare[0];
    loop {
        if ev.read().events_compare().bit_is_set() {
            ev.write(|w| w.events_compare().clear_bit());
            return;
        }
    }
}

#[interrupt]
fn TIMER0() {
    asm::nop();
    asm::nop();
    cm_interrupt::free(|cs| {
        if let Some(timer0) = EV_TIMER0.borrow(&cs).borrow().as_ref() {
            let ev = &timer0.events_compare[0]; 
            if ev.read().events_compare().bit_is_set() {
                let new_val =
                    if COUNTER.load(Relaxed) == 0 {
                        1
                    } else {
                        0
                    };
                COUNTER.store(new_val, Relaxed);
                ev.write(|w| w.events_compare().clear_bit());
            }
        }
        // if let Some(ref mut led) = LED_PIN.borrow(&cs).borrow_mut().as_mut() {
        //     led.set_high().unwrap();
        // }
    });
}

// fn wait_change_counter(prev: &mut u8) {
//     loop {
//         let current = COUNTER.load(Relaxed);
//         if current != *prev {
//             *prev = current;
//             return;
//         }
//     }
// }

fn read_vtor(cpers: &CorePeripherals) -> u32 {
    return cpers.SCB.vtor.read();
}

fn write_vtor(cpers: &CorePeripherals, vector_offset: u32) {
    cm_interrupt::free(|_| {
        unsafe { cpers.SCB.vtor.write(vector_offset); }
    });
}

#[entry]
fn main() -> ! {
    let mut cpers = CorePeripherals::take().unwrap();
    let pers = Peripherals::take().unwrap();
    let p0 = pers.P0.split();
    let timer0 = pers.TIMER0;
    let mut led = p0.p0_07.into_open_drain_output(Disconnect0Standard1, Low);
    led.set_low().unwrap();
    // let mut prev_counter = 0;

    // assert_blink(primask::read().is_active(), &mut led);
    // assert_blink(read_vtor(&cpers) == 0, &mut led);
    
    write_vtor(&cpers, 0x26000);
    
    config_timer0(&timer0, &mut cpers.NVIC, 1_000_000);
    start_timer0(&timer0);

    cm_interrupt::free(move |cs| {
        EV_TIMER0.borrow(&cs).replace(Some(timer0));
        // LED_PIN.borrow(&cs).replace(Some(led));
    });
    
    // cm_interrupt::free(|cs| {
    //     if let Some(ref mut led) = LED_PIN.borrow(&cs).borrow_mut().as_mut() {
    //         led.set_low().unwrap();
    //     }
    // });

    unsafe { cm_interrupt::enable(); }

    loop {
        asm::wfi();
        
        if COUNTER.load(Relaxed) == 0 {
            led.set_low().unwrap();
        } else {
            led.set_high().unwrap();
        }
        
        // led.set_high().unwrap();
        // wait_timer0(&pers.TIMER0);
        // wait_change_counter(&mut prev_counter);
        // led.set_low().unwrap();
        // wait_timer0(&pers.TIMER0);
        // wait_change_counter(&mut prev_counter);
    }
}
