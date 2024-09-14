use std::{
    ffi::{CStr, CString},
    fmt::Debug,
};

use crate::{prelude::*, DowncastSEXP, ProtectedSEXP};
use derive_more::Deref;
use libR_sys::*;

#[derive(Deref)]
#[repr(transparent)]
/// A wrapper around a symbol object.
pub struct Symbol<T: JustSEXP> {
    sexp: T,
}

impl<T: JustSEXP + Debug> Debug for Symbol<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol({:?})", self.sexp)
    }
}

impl Symbol<SEXP> {
    /// Create a new symbol from a C string.
    pub fn new_cstr(name: &CStr) -> Self {
        unsafe {
            let sexp = Rf_install(name.as_ptr() as *const i8);
            Symbol { sexp }
        }
    }
    /// Create a new symbol from a Rust string.
    pub fn new(name: &str) -> Self {
        let cstr = CString::new(name).expect("Failed to convert name to CString");
        Symbol::new_cstr(cstr.as_c_str())
    }
}

impl<T: JustSEXP> HasSEXP for Symbol<T> {
    fn get_sexp(&self) -> SEXP {
        self.sexp.get_sexp()
    }
}

unsafe impl<T: JustSEXP> JustSEXP for Symbol<T> {
    type Inner = T;
    fn inner_ref(&self) -> &Self::Inner {
        &self.sexp
    }
    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        Symbol {
            sexp: T::wrap_sexp_unchecked(sexp),
        }
    }
    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if unsafe { Rf_isSymbol(sexp) }.into() {
            Some(Symbol {
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

impl<T: JustSEXP> TypedSEXP for Symbol<T> {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::SYMSXP;
}

impl<T: JustSEXP> DowncastSEXP<Symbol<T>> for T {
    fn downcast(self) -> Option<Symbol<T>> {
        Symbol::wrap_sexp(self.get_sexp())
    }
}

unsafe impl<T: JustSEXP> ProtectedSEXP for Symbol<T> {
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
/// A wrapper around an environment object.
pub struct Env<T: JustSEXP> {
    env: T,
}

impl<T: JustSEXP + Debug> Debug for Env<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Env").field(&self.env).finish()
    }
}

impl Env<SEXP> {
    /// Get the global environment.
    pub fn global() -> Self {
        Env {
            env: unsafe { R_GlobalEnv },
        }
    }

    /// The static empty environment.
    pub fn empty() -> Self {
        Env {
            env: unsafe { R_EmptyEnv },
        }
    }

    /// The static base environment.
    pub fn base() -> Self {
        Env {
            env: unsafe { R_BaseEnv },
        }
    }

    /// The current environment.
    pub fn current() -> Self {
        Env {
            env: unsafe { R_GetCurrentEnv() },
        }
    }

    /// Create a new environment with the given parent, and size.
    pub fn new<P: JustSEXP>(parent: Env<P>, hashed: bool, size: i32) -> Self {
        Env {
            env: unsafe { R_NewEnv(parent.get_sexp(), hashed.into(), size) },
        }
    }
}

impl<T: JustSEXP> HasSEXP for Env<T> {
    fn get_sexp(&self) -> SEXP {
        self.env.get_sexp()
    }
}

impl<T: JustSEXP> Env<T> {
    /// Peek at a symbol in the environment.
    pub fn peek<S: JustSEXP>(&self, symbol: Symbol<S>) -> Option<SEXP> {
        unsafe {
            let sexp = Rf_findVarInFrame(self.get_sexp(), symbol.get_sexp());
            if sexp.is_null() {
                None
            } else {
                Some(sexp)
            }
        }
    }

    /// Assign a value to a symbol in the environment.
    pub fn assign<S: JustSEXP>(&mut self, symbol: Symbol<S>, value: impl HasSEXP) {
        unsafe {
            Rf_defineVar(symbol.get_sexp(), value.get_sexp(), self.get_sexp());
        }
    }
}

unsafe impl<T: JustSEXP> JustSEXP for Env<T> {
    type Inner = T;
    fn inner_ref(&self) -> &Self::Inner {
        &self.env
    }
    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        Env {
            env: T::wrap_sexp_unchecked(sexp),
        }
    }
    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if unsafe { Rf_isEnvironment(sexp) }.into() {
            Some(Env {
                env: T::wrap_sexp(sexp)?,
            })
        } else {
            None
        }
    }
    fn upcast(self) -> Self::Inner {
        self.env
    }
}

impl<T: JustSEXP> TypedSEXP for Env<T> {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::ENVSXP;
}

impl<T: JustSEXP> DowncastSEXP<Env<T>> for T {
    fn downcast(self) -> Option<Env<T>> {
        Env::wrap_sexp(self.get_sexp())
    }
}
