use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use syn::{parse_quote, Generics, WhereClause, WherePredicate};

fn add_bounds(where_clause: &mut Option<WhereClause>, generics: &Generics) {
    let bounds = generics.params.iter().filter_map(|param| match param {
        syn::GenericParam::Type(ty) => {
            Some::<WherePredicate>(parse_quote! { #ty: const_serialize::SerializeConst, })
        }
        syn::GenericParam::Lifetime(_) => None,
        syn::GenericParam::Const(_) => None,
    });
    if let Some(clause) = where_clause {
        clause.predicates.extend(bounds);
    } else {
        *where_clause = Some(parse_quote! { where #(#bounds)* });
    }
}

/// Derive the const serialize trait for a struct
#[proc_macro_derive(SerializeConst)]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(fields) => {
                let ty = &input.ident;
                let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
                let mut where_clause = where_clause.cloned();
                add_bounds(&mut where_clause, &input.generics);
                let field_names = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap());
                let field_types = fields.named.iter().map(|field| &field.ty);
                quote! {
                    unsafe impl #impl_generics const_serialize::SerializeConst for #ty #ty_generics #where_clause {
                        const ENCODING: const_serialize::Encoding = const_serialize::Encoding::Struct(const_serialize::StructEncoding::new(
                            std::mem::size_of::<Self>(),
                            &[#(
                                const_serialize::PlainOldData::new(
                                    std::mem::offset_of!(#ty, #field_names),
                                    &<#field_types as const_serialize::SerializeConst>::ENCODING,
                                ),
                            )*],
                        ));
                    }
                }.into()
            }
            syn::Fields::Unit => {
                let ty = &input.ident;
                let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
                let mut where_clause = where_clause.cloned();
                add_bounds(&mut where_clause, &input.generics);
                quote! {
                    unsafe impl #impl_generics const_serialize::SerializeConst for #ty #ty_generics #where_clause {
                        const ENCODING: const_serialize::Encoding = const_serialize::Encoding::Struct(const_serialize::StructEncoding::new(
                            std::mem::size_of::<Self>(),
                            &[],
                        ));
                    }
                }.into()
            }
            _ => syn::Error::new(
                input.ident.span(),
                "Only structs with named fields are supported",
            )
            .to_compile_error()
            .into(),
        },
        _ => syn::Error::new(input.ident.span(), "Only structs are supported")
            .to_compile_error()
            .into(),
    }
}
