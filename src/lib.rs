//! # Typed-SEXP
//!
//! Type-safe manipulation of R's SEXP objects without bells and whistles.
//!
//! ## Objectives
//!
//! R internal API is very confusing and not type-safe.
//!
//! However many times I do not want to pull in huge proc macros just to write my R extension.
//! The main reasons are:
//!   - Macros in general are hard to debug.
//!   - Macros mess up static analysis tools.
//!   - Macros hide the actual code that is being generated.
//!
//! This library is an attempt to provide a type-safe interface to R's SEXP objects without any user-exposed macros.
//! However it is not a goal to provide a high-level abstractions over R's API like `Rcpp` or `extendr`, the most I will do is type-safe indexing, attributes, calling functions, etc.
//! Users are expected to have a general understanding of R internals.
//! Here is a quick [tutorial](https://github.com/hadley/r-internals).
//!
//! ## Features
//!
//! - Typed SEXP objects that are ABI-compatible with R's SEXP.
//! - Type-safe (mutable) indexing of vectors and matrices.
//! - Typed protection of SEXP objects with a debug mode that sanity-checks the protection order.
//! - Stack RAII-based auto un-protection of SEXP objects.
//! - Downcasting of SEXP objects in a method chain with a single call (even if the object is wrapped in another abstraction).
//! - Namespace and function objects that are environment-aware.
//! - Call R functions with type-safe arguments and return values.
//! - (TODO): Dynamically create R functions.
#![warn(missing_docs)]

use libR_sys::{SEXPTYPE::*, *};

use protect::{BoxProtected, Protected};

pub use libR_sys;
pub use libR_sys::SEXP;
use sexp::{
    env::{Env, Symbol},
    function::{Builtin, Closure, Function},
    lang::{Lang, PairlistBuilder},
    matrix::Matrix,
    vector::{CharacterVector, IntegerVector, List, LogicalVector, RealVector},
};

pub(crate) mod debug;
#[cfg(feature = "embedded")]
pub mod embedded;
pub mod message;
pub mod prelude;
pub mod protect;
pub mod sexp;

/// Any supported SEXP type.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum AnySexp<T: JustSEXP> {
    Nil(T),
    Logical(LogicalVector<T>),
    LogicalMatrix(Matrix<LogicalVector<T>>),
    Real(RealVector<T>),
    RealMatrix(Matrix<RealVector<T>>),
    Integer(IntegerVector<T>),
    IntegerMatrix(Matrix<IntegerVector<T>>),
    Character(CharacterVector<T>),
    CharacterMatrix(Matrix<CharacterVector<T>>),
    List(List<T>),
    Symbol(Symbol<T>),
    Lang(Lang<T>),
    Function(Function<T>),
    Builtin(Builtin<T>),
    Closure(Closure<T>),
    Environment(Env<T>),
    Other(T),
}

impl<T: JustSEXP> From<T> for AnySexp<T> {
    fn from(value: T) -> Self {
        let sexp = value.get_sexp();
        match unsafe { TYPEOF(sexp) } {
            NILSXP => AnySexp::Nil(value),
            LGLSXP => Matrix::wrap_sexp(sexp)
                .map(AnySexp::LogicalMatrix)
                .unwrap_or_else(|| {
                    AnySexp::Logical(unsafe { LogicalVector::wrap_sexp_unchecked(sexp) })
                }),
            REALSXP => Matrix::wrap_sexp(sexp)
                .map(AnySexp::RealMatrix)
                .unwrap_or_else(|| AnySexp::Real(unsafe { RealVector::wrap_sexp_unchecked(sexp) })),
            INTSXP => Matrix::wrap_sexp(sexp)
                .map(AnySexp::IntegerMatrix)
                .unwrap_or_else(|| {
                    AnySexp::Integer(unsafe { IntegerVector::wrap_sexp_unchecked(sexp) })
                }),
            STRSXP => Matrix::wrap_sexp(sexp)
                .map(AnySexp::CharacterMatrix)
                .unwrap_or_else(|| {
                    AnySexp::Character(unsafe { CharacterVector::wrap_sexp_unchecked(sexp) })
                }),
            SYMSXP => AnySexp::Symbol(unsafe { Symbol::wrap_sexp_unchecked(sexp) }),
            LANGSXP => AnySexp::Lang(unsafe { Lang::wrap_sexp_unchecked(sexp) }),
            ENVSXP => AnySexp::Environment(unsafe { Env::wrap_sexp_unchecked(sexp) }),
            FUNSXP => AnySexp::Function(unsafe { Function::wrap_sexp_unchecked(sexp) }),
            VECSXP => AnySexp::List(unsafe { List::wrap_sexp_unchecked(sexp) }),
            _ => {
                if let Some(builtin) = Builtin::<T>::wrap_sexp(sexp) {
                    AnySexp::Builtin(builtin)
                } else if let Some(closure) = Closure::<T>::wrap_sexp(sexp) {
                    AnySexp::Closure(closure)
                } else {
                    AnySexp::Other(value)
                }
            }
        }
    }
}

impl<T: JustSEXP> AnySexp<T> {
    /// Get the inner value.
    pub fn into_inner(self) -> T {
        match self {
            AnySexp::Nil(value) => value,
            AnySexp::Logical(value) => value.upcast(),
            AnySexp::LogicalMatrix(value) => value.upcast(),
            AnySexp::Real(value) => value.upcast(),
            AnySexp::RealMatrix(value) => value.upcast(),
            AnySexp::Integer(value) => value.upcast(),
            AnySexp::IntegerMatrix(value) => value.upcast(),
            AnySexp::Character(value) => value.upcast(),
            AnySexp::CharacterMatrix(value) => value.upcast(),
            AnySexp::Symbol(value) => value.upcast(),
            AnySexp::Lang(value) => value.upcast(),
            AnySexp::Function(value) => value.upcast(),
            AnySexp::Builtin(value) => value.upcast(),
            AnySexp::Closure(value) => value.upcast(),
            AnySexp::Environment(value) => value.upcast(),
            AnySexp::List(value) => value.upcast(),
            AnySexp::Other(value) => value,
        }
    }
    /// Get a reference to the inner value.
    pub fn inner_ref(&self) -> &T {
        match self {
            AnySexp::Nil(value) => value,
            AnySexp::Logical(value) => value.inner_ref(),
            AnySexp::LogicalMatrix(value) => value.inner_ref(),
            AnySexp::Real(value) => value.inner_ref(),
            AnySexp::RealMatrix(value) => value.inner_ref(),
            AnySexp::Integer(value) => value.inner_ref(),
            AnySexp::IntegerMatrix(value) => value.inner_ref(),
            AnySexp::Character(value) => value.inner_ref(),
            AnySexp::CharacterMatrix(value) => value.inner_ref(),
            AnySexp::Symbol(value) => value.inner_ref(),
            AnySexp::Lang(value) => value.inner_ref(),
            AnySexp::Function(value) => value.inner_ref(),
            AnySexp::Builtin(value) => value.inner_ref(),
            AnySexp::Closure(value) => value.inner_ref(),
            AnySexp::Environment(value) => value.inner_ref(),
            AnySexp::List(value) => value.inner_ref(),
            AnySexp::Other(value) => value,
        }
    }
}
impl<T: JustSEXP> HasSEXP for AnySexp<T> {
    fn get_sexp(&self) -> SEXP {
        match self {
            AnySexp::Nil(value) => value.get_sexp(),
            AnySexp::Logical(value) => value.get_sexp(),
            AnySexp::LogicalMatrix(value) => value.get_sexp(),
            AnySexp::Real(value) => value.get_sexp(),
            AnySexp::RealMatrix(value) => value.get_sexp(),
            AnySexp::Integer(value) => value.get_sexp(),
            AnySexp::IntegerMatrix(value) => value.get_sexp(),
            AnySexp::Character(value) => value.get_sexp(),
            AnySexp::CharacterMatrix(value) => value.get_sexp(),
            AnySexp::Symbol(value) => value.get_sexp(),
            AnySexp::Lang(value) => value.get_sexp(),
            AnySexp::Function(value) => value.get_sexp(),
            AnySexp::Builtin(value) => value.get_sexp(),
            AnySexp::Closure(value) => value.get_sexp(),
            AnySexp::Environment(value) => value.get_sexp(),
            AnySexp::List(value) => value.get_sexp(),
            AnySexp::Other(value) => value.get_sexp(),
        }
    }
}

/// The null SEXP.
pub fn null() -> SEXP {
    unsafe { R_NilValue }
}

/// A trait for objects that have an static underlying [`SEXPTYPE`].
pub trait TypedSEXP: HasSEXP {
    /// The type of the SEXP.
    const SEXP_TYPE: SEXPTYPE;
}

/// A trait for objects that have an underlying [`SEXP`].
pub trait HasSEXP {
    /// Get the underlying SEXP.
    fn get_sexp(&self) -> SEXP;

    /// Check if the SEXP is null.
    fn is_sexp_null(&self) -> bool {
        unsafe { self.get_sexp() == libR_sys::R_NilValue }
    }

    /// Get the type of the SEXP.
    fn sexp_type(&self) -> SEXPTYPE {
        unsafe { TYPEOF(self.get_sexp()) }
    }

    /// Print the value to the R console.
    fn r_print(&self) {
        unsafe {
            Rf_PrintValue(self.get_sexp());
        }
    }

    /// Get the attribute of the SEXP.
    fn attrib(&self, tag: SEXP) -> SEXP {
        unsafe { Rf_getAttrib(self.get_sexp(), tag) }
    }

    /// Coerce the underlying SEXP to the given type.
    fn coerce(&self, sexp_type: SEXPTYPE) -> SEXP {
        unsafe { Rf_coerceVector(self.get_sexp(), sexp_type) }
    }

    /// Protect the object in the protection stack.
    fn protect(self) -> Protected<Self>
    where
        Self: Sized,
    {
        Protected::new(self)
    }

    /// Protect the object in the protection heap.
    fn protect_box(self) -> BoxProtected<Self>
    where
        Self: Sized,
    {
        BoxProtected::new(self)
    }

    /// Shorthand for `DowncastSEXP::downcast`.
    fn downcast_to<T: HasSEXP>(self) -> Option<T>
    where
        Self: DowncastSEXP<T>,
    {
        self.downcast()
    }
}

impl HasSEXP for SEXP {
    fn get_sexp(&self) -> SEXP {
        *self
    }
}

/// A [`SEXP`] wrapper that can be indexed.
pub trait IndexableSEXP: HasSEXP {
    /// The type of the index.
    type Index;
    /// The type of the output.
    type Output;
    #[must_use]
    /// The scalar length of the object.
    fn len(&self) -> usize;
    /// Check if the index is inbound.
    fn check_inbound(&self, index: usize) {
        if index >= self.len() {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                self.len(),
                index
            );
        }
    }

    /// Get the element at the given index.
    fn get_elt(&self, index: Self::Index) -> Self::Output;
    /// Set the element at the given index.
    fn set_elt(&mut self, index: Self::Index, value: impl Into<Self::Output>);
}

/// Trivial, ABI-compatible wrapper types around [`SEXP`].
///
/// Marking the wrong thing is ridiculously unsafe and should be used with caution.
///
/// It is deliberately not possible to get the inner SEXP from this type without destroying it.
/// Because the underlying assumption implementors have may be broken.
///
/// # Safety
///
/// - Both types should have the same size and alignment as [`SEXP`], and should just be a wrapper around it.
/// - [`Drop`] implementation for this type may or may not be called.
pub unsafe trait JustSEXP: HasSEXP + Sized {
    /// The inner type that this type wraps, of course it has to be also [`JustSEXP`].
    type Inner: JustSEXP;
    /// Transmute this type to another type that wraps the same [`SEXP`].
    ///
    /// # Safety
    ///
    /// Although this is memory safe, be sure that the target type is expecting the same type of SEXP.
    unsafe fn transmute_to<U: JustSEXP>(self) -> U {
        let sexp = self.get_sexp();
        std::mem::forget(self);
        U::wrap_sexp_unchecked(sexp)
    }

    /// Create a new object by wrapping a SEXP.
    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        Some(unsafe { Self::wrap_sexp_unchecked(sexp) })
    }

    /// Create a new object by wrapping a SEXP without checking the type.
    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self;

    /// Upcast this type to the inner type.
    fn upcast(self) -> Self::Inner;

    /// Get a reference to the inner type.
    fn inner_ref(&self) -> &Self::Inner;
}

unsafe impl JustSEXP for SEXP {
    type Inner = SEXP;

    fn upcast(self) -> Self::Inner {
        self
    }

    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        sexp
    }

    fn inner_ref(&self) -> &Self::Inner {
        self
    }
}

/// Trait for types whose [`SEXP`] is protected.
pub unsafe trait ProtectedSEXP: HasSEXP {
    /// The inner type that this type wraps.
    type Inner: HasSEXP;

    /// Forget the object and return it.
    fn forget(self) -> Self::Inner;

    /// Unprotect the object and return it.
    fn unprotect(self) -> Self::Inner;

    /// Start building a pairlist with this object.
    fn build_pairlist(self) -> PairlistBuilder<Self>
    where
        Self: Sized,
        Self::Inner: JustSEXP,
    {
        PairlistBuilder::new(self)
    }
}

/// Type refinement of another [`SEXP`]-wrapping type.
///
/// The [`HasSEXP::downcast_to`] method enables this trait to be used within a method chain.
///
pub trait DowncastSEXP<T: HasSEXP>: HasSEXP + Sized {
    /// Attempt to downcast the type to another type.
    fn downcast(self) -> Option<T>;
}

impl<T: HasSEXP> DowncastSEXP<T> for T {
    fn downcast(self) -> Option<T> {
        Some(self)
    }
}
