//! Common traits and types for working with this crate.

pub use crate::{
    message::UnwrapR as _,
    null as r_nil,
    sexp::{
        matrix::Matrix as MatrixSEXP,
        vector::{
            Character as CharacterSEXP, CharacterVector as CharacterVectorSEXP,
            IntegerVector as IntegerVectorSEXP, LogicalVector as LogicalVectorSEXP,
            RealVector as RealVectorSEXP,
        },
    },
    AnySexp, HasSEXP, IndexableSEXP as _, JustSEXP, ProtectedSEXP as _, TypedSEXP,
};
pub use libR_sys;
pub use libR_sys::{SEXP, SEXPTYPE};
