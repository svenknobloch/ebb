extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed};

#[proc_macro_derive(Ports)]
pub fn derive(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    (move || {
        let DeriveInput {
            vis,
            ident,
            generics,
            data,
            ..
        } = syn::parse(tokens)?;

        let (gen_impl, gen_type, gen_where) = generics.split_for_impl();

        let fields;
        match data {
            Data::Struct(DataStruct {
                fields: Fields::Named(FieldsNamed { named, .. }),
                ..
            }) => {
                fields = named
                    .into_iter()
                    .map(|Field { ident, ty, .. }| (ident.unwrap(), ty))
                    .collect::<Vec<_>>();
            }
            _ => {
                return Err(Error::new(
                    Span::call_site(),
                    "input must be a struct with named fields",
                ))
            }
        }

        let handle_ident = format_ident!("__{}Handle", ident);

        let handle_fields = fields
            .iter()
            .map(|(ident, ty)| {
                quote! { pub #ident: <#ty as ::ebb::Ports>::Handle, }
            })
            .collect::<TokenStream>();

        let values = fields
            .iter()
            .map(|(ident, ty)| {
                quote! { let #ident = <#ty as ::ebb::Ports>::create(config); }
            })
            .collect::<TokenStream>();

        let ports_values = fields
            .iter()
            .map(|(ident, _)| {
                quote! { #ident, }
            })
            .collect::<TokenStream>();

        let handle_values = fields
            .iter()
            .map(|(ident, _)| {
                quote! { #ident: ::ebb::Ports::handle(&self.#ident), }
            })
            .collect::<TokenStream>();

        Ok(quote! {

            #vis struct #handle_ident #gen_impl #gen_where {
                #handle_fields
            }

            impl #gen_impl ::ebb::Ports for #ident #gen_type #gen_where {
                type Handle = #handle_ident #gen_type;

                fn handle(&self) -> Self::Handle {
                    Self::Handle {
                        #handle_values
                    }
                }

                fn create(config: &::ebb::NetworkConfig) -> Self {
                    #values

                    Self {
                        #ports_values
                    }
                }
            }
        })
    })()
    .unwrap_or_else(|e: Error| e.to_compile_error())
    .into()
}
