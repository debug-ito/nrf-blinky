//! Cortex-M Mutex wrapper for sharing mutable data between interrupt and normal contexts.

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

    // pub fn borrow<'a>(&'a self, cs: &'a CriticalSection) -> Option<&'a T> {
    pub fn borrow<'a>(&'a self, cs: &'a CriticalSection) -> Ref<'a, Option<T>> {
        return self.0.borrow(cs).borrow();

        // self.0.borrow(cs) : &'cs RefCell<Option<T>>
        // self.0.borrow(cs).borrow() : Ref<Option<T>>    (Ref : Deref) ... このRefがtemporary objectなのか。
        // self.0.borrow(cs).borrow().deref() : &Option<T>  ... Refと同じlifetimeなはず。
        // self.0.borrow(cs).borrow().deref().as_ref() : Option<&T>
    }

    pub fn borrow_mut<'a>(&'a self, cs: &'a CriticalSection) -> RefMut<'a, Option<T>> {
        return self.0.borrow(cs).borrow_mut();
    }
}

