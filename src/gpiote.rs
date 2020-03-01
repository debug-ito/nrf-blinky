//! GPIOTE device

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::interrupt as cinterrupt;
use cortex_m::peripheral::NVIC;

use nrf52840_hal::target::{
    interrupt, Interrupt,
    GPIOTE as GPIOTE_t,
    gpiote::intenset::W as IntensetW,
    gpiote::intenclr::W as IntenclrW,
};

use crate::util::get_from_mutex;

pub struct Channels {
    pub chan0: ChanUninit,
    pub chan1: ChanUninit,
    pub chan2: ChanUninit,
    pub chan3: ChanUninit,
    pub chan4: ChanUninit,
    pub chan5: ChanUninit,
    pub chan6: ChanUninit,
    pub chan7: ChanUninit,
}

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

pub struct ConfigOut {
    pub port: u8,
    pub pin: u8,
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

impl Channels {
    pub fn from_device(dev: GPIOTE_t) -> Channels {
        cinterrupt::free(|cs| {
            DEVICE.borrow(&cs).replace(Some(dev));
        });
        return Channels {
            chan0: ChanUninit { chan: 0 },
            chan1: ChanUninit { chan: 1 },
            chan2: ChanUninit { chan: 2 },
            chan3: ChanUninit { chan: 3 },
            chan4: ChanUninit { chan: 4 },
            chan5: ChanUninit { chan: 5 },
            chan6: ChanUninit { chan: 6 },
            chan7: ChanUninit { chan: 7 },
        };
    }
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

fn set_inten(w: &mut IntensetW, chan: usize) -> &mut IntensetW {
    return match chan {
        0 => w.in0().set(),
        1 => w.in1().set(),
        2 => w.in2().set(),
        3 => w.in3().set(),
        4 => w.in4().set(),
        5 => w.in5().set(),
        6 => w.in6().set(),
        7 => w.in7().set(),
        _ => panic!("This should not happen."),
    };
}

fn clear_inten(w: &mut IntenclrW, chan: usize) -> &mut IntenclrW {
    return match chan {
        0 => w.in0().clear(),
        1 => w.in1().clear(),
        2 => w.in2().clear(),
        3 => w.in3().clear(),
        4 => w.in4().clear(),
        5 => w.in5().clear(),
        6 => w.in6().clear(),
        7 => w.in7().clear(),
        _ => panic!("This should not happen."),
    };
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
                set_inten(w, self.chan)
            });
            HANDLER[self.chan].borrow(&cs).replace(Some(config.handler));
            unsafe { NVIC::unmask(Interrupt::GPIOTE); }
        });
    }

    pub fn into_output(self, config: ConfigOut) -> ChanOut {
        cinterrupt::free(|cs| {
            match get_from_mutex(&DEVICE, &cs) {
                None => panic!("This should not happen."),
                Some(ref dev) => self.config_output(dev, config),
            }
        });
        return ChanOut { chan: self.chan };
    }

    fn config_output(&self, dev: &GPIOTE_t, config: ConfigOut) {
        dev.config[self.chan].write(|w| {
            unsafe {
                return w.mode().task()
                    .port().bit(config.port != 0)
                    .psel().bits(config.pin)
                    .polarity().toggle()
                    .outinit().low();
            }
        });
        cinterrupt::free(|_| {
            dev.intenclr.write(|w| clear_inten(w, self.chan));
        });
    }
}

enum OutCommand {Set, Clear, Out}

impl ChanOut {
    pub fn to_high(&mut self) {
        self.run_command(OutCommand::Set);
    }

    pub fn to_low(&mut self) {
        self.run_command(OutCommand::Clear);
    }

    pub fn toggle(&mut self) {
        self.run_command(OutCommand::Out);
    }

    fn run_command(&self, com: OutCommand) {
        cinterrupt::free(|cs| {
            if let Some(ref dev) = get_from_mutex(&DEVICE, &cs) {
                match com {
                    OutCommand::Set =>
                        dev.tasks_set[self.chan].write(|w| w.tasks_set().set_bit()),
                    OutCommand::Clear =>
                        dev.tasks_clr[self.chan].write(|w| w.tasks_clr().set_bit()),
                    OutCommand::Out =>
                        dev.tasks_out[self.chan].write(|w| w.tasks_out().set_bit()),
                }
            }
        });
    }
}

