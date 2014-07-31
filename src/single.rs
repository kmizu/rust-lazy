use std::cell::UnsafeCell;
use std::ptr;
use std::kinds::marker;

/// A sometimes-cleaner name for a lazily evaluated value.
pub type Lazy<T> = Thunk<T>;

/// A lazily evaluated value.
pub struct Thunk<T> {
    inner: UnsafeCell<Inner<T>>,
    noshare: marker::NoShare
}

impl<T> Thunk<T> {
    /// Create a lazily evaluated value from a proc that returns that value.
    pub fn new(producer: proc() -> T) -> Thunk<T> {
        Thunk { inner: UnsafeCell::new(Unevaluated(producer)), noshare: marker::NoShare }
    }

    /// Force evaluation of a thunk.
    pub fn force(&self) {
        unsafe {
            match *self.inner.get() {
                Evaluated(_) => return,
                EvaluationInProgress => {
                    fail!("Thunk::force called recursively. (A Thunk tried to force itself while trying to force itself).")
                },
                Unevaluated(_) => ()
            }

            match ptr::replace(self.inner.get(), EvaluationInProgress) {
                Unevaluated(producer) => *self.inner.get() = Evaluated(producer()),
                _ => unreachable!()
            };
        }
    }

    /// Force the evaluation of a thunk and get the value, consuming the thunk.
    pub fn unwrap(self) -> T {
        self.force();
        unsafe {
            match self.inner.unwrap() {
                Evaluated(val) => { val },
                _ => unreachable!()
            }
        }
    }
}

enum Inner<T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(proc() -> T)
}

impl<T> Deref<T> for Thunk<T> {
    fn deref<'a>(&'a self) -> &'a T {
        self.force();
        match unsafe { &*self.inner.get() } {
            &Evaluated(ref val) => val,
            _ => unreachable!()
        }
    }
}

impl<T> DerefMut<T> for Thunk<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        self.force();
        match unsafe { &mut *self.inner.get() } {
            &Evaluated(ref mut val) => val,
            _ => unreachable!()
        }
    }
}

