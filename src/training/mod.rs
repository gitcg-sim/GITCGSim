pub mod as_slice;

pub mod features;

#[cfg(feature = "training")]
pub mod eval;

pub mod policy;

#[cfg(not(feature = "hidden_layer"))]
pub(crate) mod hard_coded_model;

#[cfg(feature = "hidden_layer")]
pub(crate) mod hard_coded_model_hidden_layer;
