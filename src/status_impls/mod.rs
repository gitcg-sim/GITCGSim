/// Import this module to import the `StatusImpl` trait and its related imports.
#[allow(unused_imports)]
pub(crate) mod prelude {
    pub use crate::{compose_status_impls, decl_status_impl_type, list8, trigger_event_impl};
    pub use enumset::{enum_set, EnumSet, EnumSetType};

    pub use crate::ids::*;
    pub use crate::tcg_model::*;
    pub use crate::types::card_defs::Cost;
    pub use crate::types::char_state::CharStates;
    pub use crate::types::char_state::{AppliedEffectResult, AppliedEffectState};
    pub use crate::types::dice_counter::DiceDistribution;
    pub use crate::types::status_impl::RespondsTo;
    pub use crate::types::StatusSpecModifier;
    pub use crate::types::{command::*, status_impl::StatusImpl};
}

pub mod composition;

pub mod primitives;

mod static_impls;

pub use static_impls::StaticStatusImpl;

/// Macro to deduplicate StatusImpl declaration but requires nightly features to fully implement StatusImpl deduplication.
#[macro_export]
#[doc(hidden)]
macro_rules! status_impl_trait_decl {
    (
        $(#[$meta: meta])* $vis: vis trait $Name: ident {
            $(
                $(#[$fn_meta: meta])* fn $fn_name: ident ( &self $(,)? $($arg: ident: $atype: ty),* $(,)? ) -> $rtype: ty $($blk: block)? $(;)?
            )*
        }
    ) => {
        $(#[$meta])* $vis trait $Name {
            $(
                status_impl_trait_decl!(
                    @trait_method
                    $(#[$fn_meta])*; $fn_name
                    { $($arg: $atype),* };
                    $rtype
                    $(=> $blk)?
                );
            )*
        }
    };
    (
        @trait_method
        $(#[$fn_meta_m: meta])*;
        $fn_name_m: ident { $($arg_m: ident: $atype_m: ty),* }; $rtype_m: ty
    ) => {
        $(#[$fn_meta_m])* fn $fn_name_m (&self$(, $arg_m: $atype_m)*) -> $rtype_m;
    };
    (
        @trait_method
        $(#[$fn_meta_m: meta])*;
        $fn_name_m: ident { $($arg_m: ident: $atype_m: ty),* }; $rtype_m: ty => $blk_m: block
    ) => {
        $(#[$fn_meta_m])* fn $fn_name_m (&self$(, $arg_m: $atype_m)*) -> $rtype_m $blk_m
    };
}
