use std::pin::Pin;

use crate::{prelude::*, DowncastSEXP};

use derive_more::Deref;
use libR_sys::*;

#[derive(Deref)]
#[repr(transparent)]
/// A wrapper around an external pointer object.
pub struct Ptr<T: HasSEXP, W> {
    #[deref]
    ptr: T,
    _marker: std::marker::PhantomData<W>,
}

impl<T: HasSEXP, W> Ptr<T, W> {
    /// Create a new external pointer with the given pointer, tag, and protection.
    pub fn wrap<U: HasSEXP, V: JustSEXP>(ptr: *mut W, tag: V, prot: U) -> Self
    where
        T: JustSEXP,
    {
        unsafe {
            let sexp = R_MakeExternalPtr(
                ptr as *mut std::ffi::c_void,
                tag.get_sexp(),
                prot.get_sexp(),
            );
            Ptr {
                ptr: T::wrap_sexp_unchecked(sexp),
                _marker: std::marker::PhantomData,
            }
        }
    }
    /// [`wrap`] a [`Pin<Box<T>>`] into an external pointer.
    pub fn wrap_boxed<U: HasSEXP, V: JustSEXP>(ptr: Box<W>, tag: V, prot: U) -> Self
    where
        T: JustSEXP,
    {
        unsafe extern "C" fn finalize_boxed<W>(ptr: SEXP) {
            unsafe {
                drop(Box::<W>::from_raw(R_ExternalPtrAddr(ptr) as *mut W));
            }
        }
        let ret = Self::wrap(Box::into_raw(ptr).cast(), tag, prot);

        ret.register_drop(Some(finalize_boxed::<W>));

        ret
    }
    /// The SEXP of the inner protected object.
    pub fn inner_prot(&self) -> SEXP {
        unsafe { R_ExternalPtrProtected(self.ptr.get_sexp()) }
    }
    /// The SEXP of the inner tag object.
    pub fn inner_tag(&self) -> SEXP {
        unsafe { R_ExternalPtrTag(self.ptr.get_sexp()) }
    }
    /// The pointer this SEXP wraps.
    pub fn get_ptr(&self) -> *mut W {
        unsafe { R_ExternalPtrAddr(self.ptr.get_sexp()).cast() }
    }
    /// The pointer this SEXP wraps as a mutable reference.
    pub fn get_ref(&self) -> Pin<&mut W> {
        unsafe { Pin::new_unchecked(&mut *self.get_ptr()) }
    }
    /// Register a finalizer for this external pointer.
    pub fn register_drop(&self, drop: Option<unsafe extern "C" fn(SEXP)>) {
        unsafe {
            R_RegisterCFinalizerEx(self.ptr.get_sexp(), drop, Rboolean::TRUE);
        }
    }
}

impl<T: HasSEXP, W> HasSEXP for Ptr<T, W> {
    fn get_sexp(&self) -> SEXP {
        self.ptr.get_sexp()
    }
}

impl<T: JustSEXP, W> DowncastSEXP<Ptr<T, W>> for T {
    fn downcast(self) -> Option<Ptr<T, W>> {
        if self.sexp_type() == SEXPTYPE::EXTPTRSXP {
            Some(Ptr {
                ptr: self,
                _marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }
}
