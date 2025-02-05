use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(IntoSerializable)]
pub fn into_serializable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut struct_name = input.ident.to_string();
    let ref_struct_name_ident = Ident::new(struct_name.as_str(), Span::call_site());

    struct_name.truncate(struct_name.len() - 3);
    let struct_name_ident = Ident::new(struct_name.as_str(), Span::call_site());

    let fields = match input.data {
        Data::Struct(ref data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // let ref_fields = fields.iter().map(|field| {
    //     let field_name = &field.ident;
    //     let field_type = &field.ty;
    //     let ref_field_type = quote! { #field_type  };
    //     // eprintln!(
    //     //     "{} : {}",
    //     //     &field_name.as_ref().unwrap().to_string(),
    //     //     &ref_field_type.to_string()
    //     // );
    //     quote! { #field_name: #ref_field_type }
    // });
    // eprintln!("Fields");
    // for token in ref_fields {
    //     eprintln!("{}", token.to_string())
    // }
    // eprintln!("Struct fields: {:?}", ref_fields.collect::Vec<proc_macro2::TokenStream>());

    let into_serializable_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { #field_name: self.#field_name.into_serializable() }
    });

    // eprintln!("Into serializable fields");
    // for token in into_serializable_fields {
    //     eprintln!("{}", token.to_string())
    // }

    let output = quote! {
        impl IntoSerializable<#ref_struct_name_ident> for #struct_name_ident {
            fn into_serializable(&self) -> #ref_struct_name_ident {
                #ref_struct_name_ident {
                    #(#into_serializable_fields,)*
                }
            }
        }
    };
    eprintln!("{}", output.to_string());

    output.into()
}
