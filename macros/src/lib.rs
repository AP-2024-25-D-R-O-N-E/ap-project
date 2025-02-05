use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(IntoSerializable)]
pub fn into_serializable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut struct_name = input.ident.to_string();
    let ref_struct_name_ident = Ident::new(struct_name.as_str(), Span::call_site());

    struct_name.truncate(struct_name.len() - 3);
    let struct_name_ident = Ident::new(struct_name.as_str(), Span::call_site());

    match input.data {
        Data::Struct(ref data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let fields = &fields.named;

                // let ref_fields = fields.iter().map(|field| {
                //     let field_name = &field.ident;
                //     let field_type = &field.ty;
                //     let ref_field_type = quote! { #field_type  };
                //     quote! { #field_name: #ref_field_type }
                // });

                let into_serializable_fields = fields.iter().map(|field| {
                    let field_name = &field.ident;
                    quote! { #field_name: self.#field_name.into_serializable() }
                });

                let output = quote! {
                    impl IntoSerializable<#ref_struct_name_ident> for #struct_name_ident {
                        fn into_serializable(&self) -> #ref_struct_name_ident {
                            #ref_struct_name_ident {
                                #(#into_serializable_fields,)*
                            }
                        }
                    }
                };
                // eprintln!("{}", output.to_string());

                return output.into();
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
                    Fields::Named(_) => {
                        panic!("Named enums are not supported")
                    }
                    Fields::Unnamed(fields_unnamed) => {
                        let unnamed_fields =
                            fields_unnamed.unnamed.iter().enumerate().map(|(index, _)| {
                                let entry = format_ident!("par{}", index);
                                quote! {#entry}
                            });
                        let unnamed_fields_into = unnamed_fields.clone().map(|field| {
                            quote! { #field.into_serializable()}
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

                // let ref_fields_into_serializable = match &variant.fields {
                //     Fields::Named(fields) => {
                //         let named_fields = fields.named.iter().map(|field| {
                //             let field_name = field.ident.clone().unwrap();
                //             let field_type = field.ty.clone();
                //
                //             let mut field_name_string = field_name.to_string();
                //             let mut field_type_string = quote!(#field_type).to_string();
                //             field_name_string.truncate(field_name_string.len() - 3);
                //             field_type_string.truncate(field_type_string.len() - 3);
                //
                //             let field_name_ref =
                //                 Ident::new(field_name_string.as_str(), Span::call_site());
                //             let field_type_ref =
                //                 Ident::new(field_type_string.as_str(), Span::call_site());
                //
                //             quote! { #field_name_ref: #field_type_ref }
                //         });
                //         quote! { { #(#named_fields),* } }
                //     }
                //     Fields::Unnamed(fields) => {
                //
                //         let unnamed_fields = fields.unnamed.iter().enumerate().map(|(index,_)| {
                //             let entry = format_ident!("par{}", index);
                //             quote! {#entry.into_serializable()}
                //         });
                //         let rv = quote! { ::( #(#unnamed_fields),* ) };
                //         rv
                //     }
                //     Fields::Unit => quote! {},
                // };
                //
                // let rv =    quote! { #enum_name_ref_ident #variant_ident #ref_fields =>  #enum_name_ident  #variant_ident #ref_fields_into_serializable };

                // eprintln!(
                //     "Enum line: {}",
                //     rv
                // );
            }

            for line in enum_fields.iter() {
                eprintln!("{}", line);
            }

            let output = quote! {

                impl IntoSerializable<#enum_name_ident> for #enum_name_ref_ident {
                    fn into_serializable(&self) -> #enum_name_ident {
                        match self {
                            #(#enum_fields),*
                        }
                    }
                }
            };

            return output.into();
        }
        _ => panic!("Only structs and enums are supported"),
    };
}
