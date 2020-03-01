//! GPIOTE device

use nrf52840_hal::target::{
    GPIOTE as GPIOTE_t,
};

struct ChanUninit {
    port: u8,
    pin: u8
}

struct ChanIn {
    port: u8,
    pin: u8
}

struct ChanOut {
    port: u8,
    pin: u8
}

fn to_channels(dev: GPIOTE_t) -> [ChanUninit; 8] {
    // TODO
}

// user-level interrupt handlerをどうやってinterrupt handlerから叩くか？結構めんどくさいんだよな。


impl ChanUninit {
    fn config_input(self) -> ChanIn {
        // TODO
    }

    fn config_output(self) -> ChanOut {
        // TODO
    }
}
