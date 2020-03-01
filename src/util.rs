use core::ops::Deref;
use core::cell::RefCell;
use core::cell::Ref;
use cortex_m::interrupt::{Mutex, CriticalSection};

pub fn get_from_mutex<'a, T>(m: &'a Mutex<RefCell<Option<T>>>, cs: &'a CriticalSection) -> Option<Ref<'a, T>> {
    let r = m.borrow(cs).borrow();
    match r.deref() {
        Some(_) => return Some(Ref::map(r, takeout)),
        None => return None,
    };
}

fn takeout<T>(o: &Option<T>) -> &T {
    match o {
        Some(t) => return t,
        None => panic!("This is not expected."),
    }
}
