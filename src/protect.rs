//! Abstractions for protecting R objects from garbage collection.
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut, Index, IndexMut},
    sync::atomic::AtomicU32,
};

use crate::{HasSEXP, IndexableSEXP, ProtectedSEXP, TypedSEXP};
use libR_sys::*;

#[cfg(feature = "checked_protect_stack")]
use crate::debug::PROTECT_STACK_CHECK;

#[cfg(feature = "checked_protect_stack")]
use std::sync::RwLock;

/// TODO integrate with R unwind protection stack

/// Work in process: A frame of R objects on the protection stack to be unprotected together.
#[allow(unused)]
pub struct ProtectFrame {
    #[cfg(feature = "checked_protect_stack")]
    sexps: RwLock<Vec<SEXP>>,
    count: AtomicU32,
}

/// An R object that is on the protection heap.
pub struct BoxProtected<T: HasSEXP> {
    inner: Option<T>,
}

impl<T: HasSEXP> Deref for BoxProtected<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<T: HasSEXP> DerefMut for BoxProtected<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl<T: HasSEXP> BoxProtected<T> {
    /// Protect the given object in the protection heap.
    pub fn new(inner: T) -> Self {
        unsafe {
            R_PreserveObject(inner.get_sexp());
        }

        BoxProtected { inner: Some(inner) }
    }
}

impl<T: HasSEXP + Clone> Clone for BoxProtected<T> {
    fn clone(&self) -> Self {
        unsafe {
            let cloned_inner = self.inner.as_ref().unwrap().clone();
            R_PreserveObject(cloned_inner.get_sexp());
            BoxProtected {
                inner: Some(cloned_inner),
            }
        }
    }
}

impl<T: HasSEXP> HasSEXP for BoxProtected<T> {
    fn get_sexp(&self) -> SEXP {
        self.inner.as_ref().unwrap().get_sexp()
    }
}

impl<T: IndexableSEXP> IndexableSEXP for BoxProtected<T> {
    type Index = T::Index;
    type Output = T::Output;

    fn len(&self) -> usize {
        self.inner.as_ref().unwrap().len()
    }

    fn get_elt(&self, index: Self::Index) -> Self::Output {
        self.inner.as_ref().unwrap().get_elt(index)
    }

    fn set_elt(&mut self, index: Self::Index, value: impl Into<Self::Output>) {
        self.inner.as_mut().unwrap().set_elt(index, value);
    }
}

impl<T: TypedSEXP> TypedSEXP for BoxProtected<T> {
    const SEXP_TYPE: SEXPTYPE = T::SEXP_TYPE;
}

unsafe impl<T: HasSEXP> ProtectedSEXP for BoxProtected<T> {
    type Inner = T;

    fn forget(mut self) -> T {
        self.inner.take().unwrap()
    }

    fn unprotect(mut self) -> T {
        let inner = self.inner.take().unwrap();
        unsafe {
            R_ReleaseObject(inner.get_sexp());
        }
        inner
    }
}

impl<T: HasSEXP> Drop for BoxProtected<T> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            unsafe {
                R_ReleaseObject(inner.get_sexp());
            }
        }
    }
}

/// An R object that is on the protection stack.
pub struct Protected<T: HasSEXP> {
    inner: Option<T>,
}

impl<T: HasSEXP + Debug> Debug for Protected<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Protected")
            .field(self.inner.as_ref().unwrap())
            .finish()
    }
}

impl<T: HasSEXP + Display> Display for Protected<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.inner.as_ref().unwrap(), f)
    }
}

impl<T: HasSEXP> Deref for Protected<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<T: HasSEXP> Protected<T> {
    /// Protect the given object in the protection stack.
    ///
    /// The returned object must be dropped in the order it was created.
    pub fn new(inner: T) -> Self {
        unsafe {
            Rf_protect(inner.get_sexp());
            #[cfg(feature = "checked_protect_stack")]
            PROTECT_STACK_CHECK.push(inner.get_sexp());
        }

        Protected { inner: Some(inner) }
    }
}

impl<T: HasSEXP + Clone> Clone for Protected<T> {
    fn clone(&self) -> Self {
        unsafe {
            let cloned_inner = self.inner.as_ref().unwrap().clone();
            Rf_protect(cloned_inner.get_sexp());
            #[cfg(feature = "checked_protect_stack")]
            PROTECT_STACK_CHECK.push(cloned_inner.get_sexp());
            Protected {
                inner: Some(cloned_inner),
            }
        }
    }
}

impl<T: HasSEXP> Drop for Protected<T> {
    fn drop(&mut self) {
        unsafe {
            Rf_unprotect(1);
            #[cfg(feature = "checked_protect_stack")]
            PROTECT_STACK_CHECK.checked_pop(self.inner.get_sexp());
        }
    }
}

impl<T: HasSEXP> HasSEXP for Protected<T> {
    fn get_sexp(&self) -> SEXP {
        self.inner.as_ref().unwrap().get_sexp()
    }
}

impl<T: IndexableSEXP> IndexableSEXP for Protected<T> {
    type Index = T::Index;
    type Output = T::Output;

    fn len(&self) -> usize {
        self.inner.as_ref().unwrap().len()
    }

    fn get_elt(&self, index: Self::Index) -> Self::Output {
        self.inner.as_ref().unwrap().get_elt(index)
    }

    fn set_elt(&mut self, index: Self::Index, value: impl Into<Self::Output>) {
        self.inner.as_mut().unwrap().set_elt(index, value);
    }
}

impl<Idx, T: IndexableSEXP + Index<Idx>> Index<Idx> for Protected<T> {
    type Output = <T as Index<Idx>>::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.inner.as_ref().unwrap()[index]
    }
}

impl<Idx, T: IndexableSEXP + IndexMut<Idx>> IndexMut<Idx> for Protected<T> {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.inner.as_mut().unwrap()[index]
    }
}

unsafe impl<T: HasSEXP> ProtectedSEXP for Protected<T> {
    type Inner = T;

    fn forget(mut self) -> T {
        self.inner.take().unwrap()
    }

    fn unprotect(mut self) -> T {
        let inner = self.inner.take().unwrap();
        unsafe {
            Rf_unprotect(1);
            #[cfg(feature = "checked_protect_stack")]
            PROTECT_STACK_CHECK.checked_pop(inner.get_sexp());
        }
        inner
    }
}

impl<T: TypedSEXP> TypedSEXP for Protected<T> {
    const SEXP_TYPE: SEXPTYPE = T::SEXP_TYPE;
}
