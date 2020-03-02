//! Cortex-M Mutex wrapper for sharing mutable data between interrupt and normal contexts.

use core::ops::Deref;
use core::cell::{RefCell, Ref, RefMut};
use cortex_m::interrupt::{Mutex, CriticalSection};

pub struct CMutex<T> (Mutex<RefCell<Option<T>>>);

impl<T> CMutex<T> {
    pub const fn new() -> Self {
        return CMutex(Mutex::new(RefCell::new(None)));
    }

    pub fn replace(&self, cs: &CriticalSection, new_item: T) -> Option<T> {
        return self.0.borrow(cs).replace(Some(new_item));
    }

    pub fn borrow<'a>(&'a self, cs: &'a CriticalSection) -> Option<Ref<'a, T>> {
        let r = self.0.borrow(cs).borrow();
        match r.deref() {
            Some(_) => return Some(Ref::map(r, takeout)),
            None => return None,
        }
    }

    pub fn borrow_mut<'a>(&'a self, cs: &'a CriticalSection) -> RefMut<'a, Option<T>> {
        return self.0.borrow(cs).borrow_mut();
    }
}

fn takeout<T>(o: &Option<T>) -> &T {
    match o {
        Some(t) => return t,
        None => panic!("This is not expected."),
    }
}
