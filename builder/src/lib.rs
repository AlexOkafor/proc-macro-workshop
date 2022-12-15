use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Field};

#[proc_macro_derive(Builder, attributes(builder))]
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
        if ty_inner_type("Option", ty).is_some() || builder_of(f).is_some() {
            quote! { #name:#ty}
        } else {
            quote! { #name: std::option::Option<#ty>}
        }
    });

    let build_empty = fields.iter().map(|f| {
        let name = &f.ident;
        if builder_of(f).is_some() {
            quote! { #name: std::vec::Vec::new() }
        } else {
            quote! { #name: std::option::Option::None }
        }
    });

    let builder_setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let (arg_type, value) = if let Some(inner_ty) = ty_inner_type("Option", ty) {
            (inner_ty, quote! {std::option::Option::Some(#name)})
        } else {
            (ty, quote! {std::option::Option::Some(#name)})
        };
        quote! {
            fn #name(&mut self, #name:#arg_type) ->&mut Self {
                self.#name = #value;
                self
            }
        }
    });

    let build_checks = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if ty_inner_type("Option", ty).is_some() {
            quote! {}
        } else {
            quote! {
                if self.#name.is_none() {
                    // Box::new(format!("{} is not set",stringify!(#name))
                    let str = format!("{} is not set",stringify!(#name));
                    return std::result::Result::Err(String::from(str).into());
                }
            }
        }
    });

    let build_cmd_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if ty_inner_type("Option", ty).is_some() {
            quote! {#name: self.#name.clone()}
        } else {
            quote! {
                #name: self.#name.take().unwrap()
            }
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

fn ty_inner_type<'a>(wrapper: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(ref p) = ty {
        if p.path.segments.len() != 1 || p.path.segments[0].ident != wrapper {
            return None;
        }
        if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments {
            if inner_ty.args.len() != 1 {
                return None;
            }
            let inner_ty = inner_ty.args.first().unwrap();
            if let syn::GenericArgument::Type(ref t) = inner_ty {
                return Some(t);
            }
        }
    }
    None
}

fn builder_of(f: &syn::Field) -> Option<&syn::Attribute> {
    for a in &f.attrs {
        if a.path.segments.len() == 1 && a.path.segments[0].ident == "builder" {
            return Some(a);
        }
    }
    None
}
