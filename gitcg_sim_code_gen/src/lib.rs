extern crate convert_case;
extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use std::borrow::Borrow;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, DataEnum, DeriveInput};

fn iter_variants_snake_case(data_enum: &DataEnum) -> impl Iterator<Item = (&Ident, Ident)> {
    data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let mod_name = Ident::new(
            variant_name.to_string().to_case(Case::Snake).borrow(),
            variant_name.span(),
        );
        (variant_name, mod_name)
    })
}

fn for_each_enum<R: IntoIterator<Item = S>, S: quote::ToTokens>(
    data_enum: &DataEnum,
    f: impl Fn(&Ident, &Ident) -> R,
) -> proc_macro2::TokenStream {
    let mut pats = proc_macro2::TokenStream::new();
    for (variant_name, mod_name) in iter_variants_snake_case(data_enum) {
        pats.append_all(f(variant_name, &mod_name));
    }

    quote!(#pats)
}

/// Derives auto-generated code for the `CharId` enum type:
///  - Method `__generated_lookup_char_card`
///  - Method `__generated_lookup_skills`
///  - Module `__generated_char_reexports`
#[proc_macro_derive(CharIdDerives)]
pub fn char_id_derives(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let syn::Data::Enum(data_enum) = &input.data else {
        return syn::Error::new(Span::call_site(), "must be an enum")
            .to_compile_error()
            .into();
    };

    let char_card_lookup = for_each_enum(data_enum, |variant_name, mod_name| {
        quote! {
            #name::#variant_name => { & crate::cards::characters::#mod_name::C },
        }
    });

    let skills_lookup = for_each_enum(data_enum, |variant_name, mod_name| {
        quote! {
            #name::#variant_name => { & crate::cards::characters::#mod_name::SKILLS },
        }
    });

    let char_reexports = for_each_enum(data_enum, |_, mod_name| {
        quote! {
            pub use crate::cards::characters::#mod_name::*;
        }
    });

    quote! {
        impl #name {
            #[doc(hidden)]
            pub(crate) const fn __generated_lookup_char_card(self) -> &'static crate::types::card_defs::CharCard {
                match self { #char_card_lookup }
            }

            #[doc(hidden)]
            pub(crate) const fn __generated_lookup_skills(self) -> &'static [(crate::ids::enums::SkillId, crate::types::card_defs::Skill)] {
                match self { #skills_lookup }
            }
        }

        #[doc(hidden)]
        pub(crate) mod __generated_char_reexports {
            #char_reexports
        }
    }.into()
}

/// Derives generated methods for status-related enums:
/// - Method `__generated_lookup_impl`
/// - Method `__generated_lookup_status`
#[proc_macro_derive(StatusIdDerives)]
pub fn status_id_derives(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let syn::Data::Enum(data_enum) = &input.data else {
        return syn::Error::new(Span::call_site(), "must be an enum")
            .to_compile_error()
            .into();
    };

    let status_lookup_reexports_name = Ident::new(
        format!("__generated_status_lookup_reexports_for_{name}").as_str(),
        name.span(),
    );

    let impl_lookup = for_each_enum(data_enum, |variant_name, mod_name| {
        quote! {
            #name::#variant_name => { & #status_lookup_reexports_name::#mod_name::I },
        }
    });

    let status_lookup = for_each_enum(data_enum, |variant_name, mod_name| {
        quote! {
            #name::#variant_name => { & #status_lookup_reexports_name::#mod_name::S },
        }
    });

    quote! {
        #[doc(hidden)]
        mod #status_lookup_reexports_name {
            pub use super::__generated_char_reexports::*;
            pub use crate::cards::{all_cards_reexports::*, statuses::*, summons::*};
        }

        impl #name {
            #[doc(hidden)]
            pub(crate) const fn __generated_lookup_impl(self) -> &'static crate::types::status_impl::StatusImpl {
                match self { #impl_lookup }
            }

            #[doc(hidden)]
            pub(crate) const fn __generated_lookup_status(self) -> &'static crate::types::card_defs::Status {
                match self { #status_lookup }
            }
        }
    }
    .into()
}

#[proc_macro_derive(GetStatus)]
pub fn status_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    quote! {
        impl crate::ids::lookup::GetStatus for #name {
            #[inline]
            fn status(self) -> &'static crate::types::card_defs::Status {
                self.__generated_lookup_status()
            }

            #[inline]
            fn status_impl(self) -> &'static dyn crate::types::status_impl::StatusImpl {
                self.__generated_lookup_impl()
            }
        }
    }
    .into()
}

/// Derives the `__generated_enum_cases_$enum_name` macro, with the following definition where
/// `$case_name` is the name of the enum case and `$module_name` is the name of the module
/// corresponding to the enum case.
///
/// ```rust,ignore
/// macro_rules! __generated_enum_cases_$name {
///     ($expr: expr, &$I: ident $(|$val: ident| $blk: block $(,)? )?) => {
///         match $expr {
///             $name::$case_name => {
///                 let $val = &($module_name::$I);
///                 $blk
///                 // or replace the entire block with $val itself if $val and $blk are not provided
///             },
///             ...
///         }
///     }
/// }
/// ```
///
/// This generated macro is used to generate non-dynamic (no `dyn`) `StatusImpl` implementations.
#[proc_macro_derive(GeneratedEnumCasesMacro)]
pub fn macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let syn::Data::Enum(data_enum) = &input.data else {
        return syn::Error::new(Span::call_site(), "must be an enum")
            .to_compile_error()
            .into();
    };
    let macro_name = Ident::new(format!("__generated_enum_cases_{name}_internal").as_str(), name.span());
    let macro_name_export = Ident::new(format!("__generated_enum_cases_{name}").as_str(), name.span());
    let macro_cases = for_each_enum(data_enum, |variant_name, mod_name| {
        quote! {
            #name::#variant_name => { $crate::__mapping!( & #mod_name::$I , $(| $val | $blk )? ) },
        }
    });
    quote! {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! #macro_name {
            ($expr: expr, & $I: ident $(, | $val: ident | $blk: block $(,)?)?) => {
                match $expr { #macro_cases }
            }
        }

        // Required to make the macro visible to rest of the crate
        #[doc(hidden)]
        pub use #macro_name as #macro_name_export;
    }
    .into()
}

#[proc_macro_derive(GetCard)]
pub fn card_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let syn::Data::Enum(data_enum) = &input.data else {
        return syn::Error::new(Span::call_site(), "must be an enum")
            .to_compile_error()
            .into();
    };

    let card_lookup = for_each_enum(data_enum, |variant_name, mod_name| {
        quote! {
            #name::#variant_name => { & crate::cards::all_cards_reexports::#mod_name::C },
        }
    });
    quote! {
        impl crate::ids::lookup::GetCard for #name {
            #[inline]
            fn card(self) -> &'static crate::types::card_defs::Card {
                match self { #card_lookup }
            }
        }
    }
    .into()
}
