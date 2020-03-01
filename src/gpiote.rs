//! GPIOTE device

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::interrupt as cinterrupt;
use cortex_m::peripheral::NVIC;

use nrf52840_hal::target::{
    interrupt, Interrupt,
    GPIOTE as GPIOTE_t,
};

use crate::util::get_from_mutex;


pub struct ChanUninit {
    chan: usize,
}

pub struct ChanIn {
    chan: usize,
}

pub struct ChanOut {
    chan: usize,
}

pub type InputHandler = fn() -> ();

pub struct ConfigIn {
    pub port: u8,
    pub pin: u8,
    pub handler: InputHandler,
}

const CHAN_NUM: usize = 8;

static DEVICE: Mutex<RefCell<Option<GPIOTE_t>>> = Mutex::new(RefCell::new(None));
static HANDLER: [Mutex<RefCell<Option<InputHandler>>>; CHAN_NUM] =
    //// This initialization cannot be used if the type doesn't have Copy trait.
    // [Mutex::new(RefCell::new(None)); CHAN_NUM];
    [
        Mutex::new(RefCell::new(None)),
        Mutex::new(RefCell::new(None)),
        Mutex::new(RefCell::new(None)),
        Mutex::new(RefCell::new(None)),
        Mutex::new(RefCell::new(None)),
        Mutex::new(RefCell::new(None)),
        Mutex::new(RefCell::new(None)),
        Mutex::new(RefCell::new(None)),
    ];


pub fn to_channels(dev: GPIOTE_t) -> [ChanUninit; 8] {
    cinterrupt::free(|cs| {
        DEVICE.borrow(&cs).replace(Some(dev));
    });
    return [
        ChanUninit { chan: 0 },
        ChanUninit { chan: 1 },
        ChanUninit { chan: 2 },
        ChanUninit { chan: 3 },
        ChanUninit { chan: 4 },
        ChanUninit { chan: 5 },
        ChanUninit { chan: 6 },
        ChanUninit { chan: 7 },
    ];
}


#[interrupt]
fn GPIOTE() {
    cinterrupt::free(|cs| {
        if let Some(dev) = get_from_mutex(&DEVICE, &cs) {
            for chan_index in 0 .. CHAN_NUM {
                let ev = &dev.events_in[chan_index];
                if ev.read().events_in().bit_is_set() {
                    ev.write(|w| w.events_in().clear_bit());
                    if let Some(handler) = get_from_mutex(&HANDLER[chan_index], &cs) {
                        handler();
                    }
                }
            }
        }
    });
}

impl ChanUninit {
    pub fn into_input(self, config: ConfigIn) -> ChanIn {
        cinterrupt::free(|cs| {
            match get_from_mutex(&DEVICE, &cs) {
                None => panic!("This should not happen."),
                Some(ref dev) => self.config_input(dev, config),
            }
        });
        return ChanIn { chan: self.chan };
    }

    fn config_input(&self, dev: &GPIOTE_t, config: ConfigIn) {
        dev.config[self.chan].write(|w| {
            unsafe {
                return w.mode().event()
                    .port().bit(config.port != 0)
                    .psel().bits(config.pin)
                    .polarity().hi_to_lo();
            }
        });
        cinterrupt::free(|cs| {
            dev.intenset.write(|w| {
                w.in0().set()
            });
            HANDLER[self.chan].borrow(&cs).replace(Some(config.handler));
            unsafe { NVIC::unmask(Interrupt::GPIOTE); }
        });
    }

    // fn config_output(self) -> ChanOut {
    //     // TODO
    // }
}

