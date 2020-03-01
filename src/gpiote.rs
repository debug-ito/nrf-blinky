//! GPIOTE device

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

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

const CHAN_NUM: usize = 8;
type InputHandler = fn() -> ();
static DEVICE: Mutex<RefCell<Option<GPIOTE_t>>> = Mutex::new(RefCell::new(None));
static HANDLER: [Mutex<RefCell<Option<InputHandler>>>; CHAN_NUM] = [Mutex::new(RefCell::new(None)); CHAN_NUM];

// Copy traitを実装していないとこのやり方の配列初期化ができない？


// fn to_channels(dev: GPIOTE_t) -> [ChanUninit; 8] {
//     // TODO
// }

// impl ChanUninit {
//     fn config_input(self) -> ChanIn {
//         // TODO
//     }
// 
//     fn config_output(self) -> ChanOut {
//         // TODO
//     }
// }
