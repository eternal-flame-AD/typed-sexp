use std::fmt::Debug;

use crate::{prelude::*, DowncastSEXP, ProtectedSEXP};
use derive_more::Deref;
use libR_sys::Rf_isFunction;

#[derive(Deref)]
#[repr(transparent)]
/// A wrapper around a builtin function object.
pub struct Builtin<T: JustSEXP> {
    sexp: T,
}

impl<T: JustSEXP + Debug> Debug for Builtin<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Builtin").field(&self.sexp).finish()
    }
}

impl<T: JustSEXP> HasSEXP for Builtin<T> {
    fn get_sexp(&self) -> SEXP {
        self.sexp.get_sexp()
    }
}

impl<T: JustSEXP> DowncastSEXP<Builtin<T>> for T {
    fn downcast(self) -> Option<Builtin<T>> {
        Builtin::wrap_sexp(self.get_sexp())
    }
}

unsafe impl<T: JustSEXP> JustSEXP for Builtin<T> {
    type Inner = T;
    fn inner_ref(&self) -> &Self::Inner {
        &self.sexp
    }
    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        Builtin {
            sexp: T::wrap_sexp_unchecked(sexp),
        }
    }
    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if unsafe { Rf_isFunction(sexp) }.into() {
            Some(Builtin {
                sexp: T::wrap_sexp(sexp)?,
            })
        } else {
            None
        }
    }
    fn upcast(self) -> Self::Inner {
        self.sexp
    }
}

unsafe impl<T: JustSEXP> ProtectedSEXP for Builtin<T> {
    type Inner = Self;

    fn forget(self) -> Self::Inner {
        self
    }

    fn unprotect(self) -> Self::Inner {
        self
    }
}

#[derive(Deref)]
#[repr(transparent)]
/// A wrapper around a function object.
pub struct Function<T: JustSEXP> {
    sexp: T,
}

impl<T: JustSEXP + Debug> Debug for Function<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Function").field(&self.sexp).finish()
    }
}

impl<T: JustSEXP> HasSEXP for Function<T> {
    fn get_sexp(&self) -> SEXP {
        self.sexp.get_sexp()
    }
}

impl<T: JustSEXP> DowncastSEXP<Function<T>> for T {
    fn downcast(self) -> Option<Function<T>> {
        Function::wrap_sexp(self.get_sexp())
    }
}

unsafe impl<T: JustSEXP> JustSEXP for Function<T> {
    type Inner = T;
    fn inner_ref(&self) -> &Self::Inner {
        &self.sexp
    }
    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        Function {
            sexp: T::wrap_sexp_unchecked(sexp),
        }
    }
    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if unsafe { Rf_isFunction(sexp) }.into() {
            Some(Function {
                sexp: T::wrap_sexp(sexp)?,
            })
        } else {
            None
        }
    }
    fn upcast(self) -> Self::Inner {
        self.sexp
    }
}

#[derive(Deref)]
#[repr(transparent)]
/// A wrapper around a closure object.
pub struct Closure<T: JustSEXP> {
    sexp: T,
}

impl<T: JustSEXP> Closure<T> {
    /// Create a new closure object.
    pub fn new(sexp: T) -> Self {
        Closure { sexp }
    }
}

impl<T: JustSEXP + Debug> Debug for Closure<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Closure").field(&self.sexp).finish()
    }
}

impl<T: JustSEXP> HasSEXP for Closure<T> {
    fn get_sexp(&self) -> SEXP {
        self.sexp.get_sexp()
    }
}

impl<T: JustSEXP> DowncastSEXP<Closure<T>> for T {
    fn downcast(self) -> Option<Closure<T>> {
        Closure::wrap_sexp(self.get_sexp())
    }
}

unsafe impl<T: JustSEXP> JustSEXP for Closure<T> {
    type Inner = T;
    fn inner_ref(&self) -> &Self::Inner {
        &self.sexp
    }
    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        Closure {
            sexp: T::wrap_sexp_unchecked(sexp),
        }
    }
    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if unsafe { Rf_isFunction(sexp) }.into() {
            Some(Closure {
                sexp: T::wrap_sexp(sexp)?,
            })
        } else {
            None
        }
    }
    fn upcast(self) -> Self::Inner {
        self.sexp
    }
}

#[derive(Deref)]
#[repr(transparent)]
/// A wrapper around a callable object.
pub struct Callable<T: JustSEXP> {
    sexp: T,
}

impl<T: JustSEXP> HasSEXP for Callable<T> {
    fn get_sexp(&self) -> SEXP {
        self.sexp.get_sexp()
    }
}

impl<T: JustSEXP> DowncastSEXP<Callable<T>> for T {
    fn downcast(self) -> Option<Callable<T>> {
        Callable::wrap_sexp(self.get_sexp())
    }
}

unsafe impl<T: JustSEXP> JustSEXP for Callable<T> {
    type Inner = T;
    fn inner_ref(&self) -> &Self::Inner {
        &self.sexp
    }
    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        Callable {
            sexp: T::wrap_sexp_unchecked(sexp),
        }
    }
    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if let Some(builtin) = Builtin::<T>::wrap_sexp(sexp) {
            Some(Callable {
                sexp: builtin.upcast(),
            })
        } else if let Some(closure) = Closure::<T>::wrap_sexp(sexp) {
            Some(Callable {
                sexp: closure.upcast(),
            })
        } else {
            Function::<T>::wrap_sexp(sexp).map(|function| Callable {
                sexp: function.upcast(),
            })
        }
    }
    fn upcast(self) -> Self::Inner {
        self.sexp
    }
}
