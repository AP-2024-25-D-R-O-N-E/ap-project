use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(AsSerializable)]
pub fn as_serializable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut struct_name = input.ident.to_string();
    let ref_struct_name_ident = Ident::new(struct_name.as_str(), Span::call_site());

    struct_name.truncate(struct_name.len() - 3);
    let struct_name_ident = Ident::new(struct_name.as_str(), Span::call_site());

    match input.data {
        Data::Struct(ref data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let fields = &fields.named;

                let into_serializable_fields = fields.iter().map(|field| {
                    let field_name = &field.ident;
                    quote! { #field_name: self.#field_name.as_serializable() }
                });

                let output = quote! {
                    impl AsSerializable for #struct_name_ident {
                        type Output = #ref_struct_name_ident;
                        fn as_serializable(&self) -> Self::Output {
                            Self::Output {
                                #(#into_serializable_fields,)*
                            }
                        }
                    }
                };
                // eprintln!("{}", output.to_string());

                output.into()
            }
            _ => panic!("Only named fields are supported"),
        },
        Data::Enum(ref data_enum) => {
            let enum_name_ident = input.ident;
            let mut enum_name_string = enum_name_ident.to_string();
            enum_name_string.truncate(enum_name_string.len() - 3);
            let enum_name_ref_ident = Ident::new(&enum_name_string, Span::call_site());

            let variants = &data_enum.variants;
            let mut enum_fields = vec![];

            for variant in variants {
                let variant_ident = &variant.ident;

                // let mut variant_string = variant_ident.to_string();
                // variant_string.truncate(variant_string.len() - 3);
                // let variant_ref_ident = Ident::new(variant_string.as_str(), Span::call_site());

                match &variant.fields {
                    Fields::Named(fields_named) => {
                        let named_fields = fields_named.named.iter().map(|field| {
                            let ident = field.ident.clone().unwrap();
                            let rv = quote! { #ident };
                            rv
                        });
                        let expanded_fields_names = quote! {  #(#named_fields),*  };

                        let named_fields_into = fields_named.named.iter().map(|field| {
                            let ident = field.ident.clone().unwrap();
                            let rv = quote! { #ident : #ident.as_serializable() };
                            rv
                        });
                        let expanded_fields_names_into = quote! {  #(#named_fields_into),*  };

                        let rv = quote! { #enum_name_ref_ident :: #variant_ident {#expanded_fields_names} =>  #enum_name_ident :: #variant_ident {#expanded_fields_names_into} };
                        // eprintln!("{}", rv);
                        enum_fields.push(rv);
                        // panic!("Named enums are not supported")
                    }
                    Fields::Unnamed(fields_unnamed) => {
                        let unnamed_fields =
                            fields_unnamed.unnamed.iter().enumerate().map(|(index, _)| {
                                let entry = format_ident!("par{}", index);
                                quote! {#entry}
                            });
                        let unnamed_fields_into = unnamed_fields.clone().map(|field| {
                            quote! { #field.as_serializable()}
                        });

                        let expanded_fields = quote! {  #(#unnamed_fields),*  };
                        let expanded_fields_into = quote! {  #(#unnamed_fields_into),*  };
                        let rv = quote! { #enum_name_ref_ident :: #variant_ident (#expanded_fields) =>  #enum_name_ident :: #variant_ident (#expanded_fields_into) };
                        enum_fields.push(rv);
                    }
                    Fields::Unit => {
                        let rv = quote! { #enum_name_ref_ident :: #variant_ident  =>  #enum_name_ident :: #variant_ident  };
                        enum_fields.push(rv);
                    }
                };
            }

            let output = quote! {

                impl AsSerializable for #enum_name_ref_ident {
                    type Output = #enum_name_ident;
                    fn as_serializable(&self) -> Self::Output {
                        match self {
                            #(#enum_fields),*
                        }
                    }
                }
            };

            // eprintln!("{}", output);
            output.into()
        }
        _ => panic!("Only structs and enums are supported"),
    }
}
