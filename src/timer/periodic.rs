//! Periodic timer

use nrf52840_hal::target::{
    interrupt, Interrupt,
    Peripherals, TIMER0 as TIMER0_t,
};

/// Periodic timer.
pub struct TimerPeriodic;

type TimerHandler = fn() -> ();

// TODO: add static variables that keep the device and handlers.


impl TimerPeriodic {
    pub fn from_device(dev: TIMER0_t) -> TimerPeriodic {
        // TODO
        return TimerPeriodic;
    }

    pub fn add_handler(&mut self, handler: TimerHandler) {
        // TODO
    }

    pub fn set_period(&mut self, period_usec: u32) {
        // TODO
    }

    pub fn start(&mut self) {
        // TODO
    }

    pub fn stop(&mut self) {
        // TODO
    }
}
