use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;
    let ast: DeriveInput = syn::parse(input).unwrap();

    derive_impl(&ast)
}

fn derive_impl(ast: &DeriveInput) -> TokenStream {
    let ident = &ast.ident;
    let builder_ident = format_ident!("{}Builder", ident);
    // let builder_ident = Ident::new(&new_name, ident.span());

    let fields = if let syn::Data::Struct(st) = &ast.data {
        if let syn::Fields::Named(names) = &st.fields {
            &names.named
        } else {
            unimplemented!();
        }
    } else {
        unimplemented!();
    };

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #name: std::option::Option<#ty>}
    });

    let build_empty = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: std::option::Option::None }
    });

    let builder_setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            fn #name(&mut self, #name:#ty) ->&mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let build_checks = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            if self.#name.is_none() {
                // Box::new(format!("{} is not set",stringify!(#name))
                let str = format!("{} is not set",stringify!(#name));
                return std::result::Result::Err(String::from(str).into());
            }
        }
    });

    let build_cmd_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: self.#name.take().unwrap()
        }
    });

    let builder_gen = quote! {
        pub struct #builder_ident {
            #(#build_fields,)*
        }

        // setters
        impl #builder_ident {
            #(#builder_setters)*
        }

        // build method
        impl #builder_ident {
            pub fn build(&mut self) -> Result<#ident,std::boxed::Box<dyn std::error::Error>> {
                #(#build_checks)*
                std::result::Result::Ok(#ident {
                    #(#build_cmd_fields,)*
                })
            }
        }
    };

    let gen = quote! {
        #builder_gen
        impl #ident {
            pub fn builder() -> #builder_ident{
                #builder_ident {
                    #(#build_empty,)*
                }
            }
        }
    };
    gen.into()
}
