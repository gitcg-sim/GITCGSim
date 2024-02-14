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

#[proc_macro_derive(StatusIdDerives)]
pub fn status_id_derives(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let syn::Data::Enum(data_enum) = &input.data else {
        return syn::Error::new(Span::call_site(), "must be an enum")
            .to_compile_error()
            .into();
    };

    let impl_lookup = for_each_enum(data_enum, |variant_name, mod_name| {
        quote! {
            #name::#variant_name => { & #mod_name::I },
        }
    });

    let status_lookup = for_each_enum(data_enum, |variant_name, mod_name| {
        quote! {
            #name::#variant_name => { & #mod_name::S },
        }
    });

    quote! {
        impl #name {
            #[doc(hidden)]
            pub(crate) const fn __generated_lookup_impl(self) -> &'static crate::types::status_impl::StatusImpl {
                use self::__generated_char_reexports::*;
                use crate::cards::{all::*, statuses::*, summons::*};
                match self { #impl_lookup }
            }

            #[doc(hidden)]
            pub(crate) const fn __generated_lookup_status(self) -> &'static crate::types::card_defs::Status {
                use self::__generated_char_reexports::*;
                use crate::cards::{all::*, statuses::*, summons::*};
                match self { #status_lookup }
            }
        }
    }
    .into()
}

#[proc_macro_derive(GetStatus)]
pub fn get_status_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    quote! {
        impl crate::ids::lookup::GetStatus for #name {
            #[inline]
            fn get_status(self) -> &'static crate::types::card_defs::Status {
                self.__generated_lookup_status()
            }

            #[inline]
            fn get_status_impl(self) -> &'static dyn crate::types::status_impl::StatusImpl {
                self.__generated_lookup_impl()
            }
        }
    }
    .into()
}

#[proc_macro_derive(GetCard)]
pub fn get_card_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let syn::Data::Enum(data_enum) = &input.data else {
        return syn::Error::new(Span::call_site(), "must be an enum")
            .to_compile_error()
            .into();
    };

    let card_lookup = for_each_enum(data_enum, |variant_name, mod_name| {
        quote! {
            #name::#variant_name => { & crate::cards::all::#mod_name::C },
        }
    });
    quote! {
        impl crate::ids::lookup::GetCard for #name {
            #[inline]
            fn get_card(self) -> &'static crate::types::card_defs::Card {
                match self { #card_lookup }
            }
        }
    }
    .into()
}
