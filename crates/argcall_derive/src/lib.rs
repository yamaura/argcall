use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, LitStr, Variant};

#[derive(Debug, Copy, Clone)]
enum CallableType {
    Callable,
    CallableMut,
    CallableOnce,
}

impl CallableType {
    fn as_trait(&self) -> TokenStream {
        match self {
            CallableType::Callable => quote! { argcall::Callable },
            CallableType::CallableMut => quote! { argcall::CallableMut },
            CallableType::CallableOnce => quote! { argcall::CallableOnce },
        }
    }

    fn as_fn(&self) -> TokenStream {
        match self {
            CallableType::Callable => quote! { call_fn(&self, _: ()) },
            CallableType::CallableMut => quote! { call_fn_mut(&mut self, _: ()) },
            CallableType::CallableOnce => quote! { call_fn_once(self, _: ()) },
        }
    }
}

/// A procedural macro to derive the Callable trait
#[proc_macro_derive(Callable, attributes(argcall))]
pub fn callable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    generic_callable(CallableType::Callable, input)
}

#[proc_macro_derive(CallableMut, attributes(argcall))]
pub fn callable_mut_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    generic_callable(CallableType::CallableMut, input)
}

#[proc_macro_derive(CallableOnce, attributes(argcall))]
pub fn callable_once_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    generic_callable(CallableType::CallableOnce, input)
}

fn generic_callable(callable_type: CallableType, input: DeriveInput) -> proc_macro::TokenStream {
    // Parse the input token stream as a DeriveInput struct

    // Get the enum name
    let enum_name = input.ident;

    // Extract the data of the enum (expecting variants)
    let data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("#[derive(Callable)] can only be applied to enums"),
    };

    let output_type = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("argcall"))
        .map(parse_output_attribute)
        .next()
        .expect("Expected #[argcall(output=...)] attribute on enum")
        .unwrap();

    let mut variant_structs = Vec::new();
    let mut match_arms = Vec::new();

    data.variants
        .iter()
        .try_for_each(|variant| {
            let (variant_struct, match_arm) =
                parse_variant(callable_type, &enum_name, &output_type, variant)?;
            variant_structs.push(variant_struct);
            match_arms.push(match_arm);
            Ok::<(), syn::Error>(())
        })
        .unwrap();

    let trait_name = callable_type.as_trait();
    let fn_type = callable_type.as_fn();

    let expanded = quote! {
        #(#variant_structs)*

        impl #trait_name for #enum_name {
            type Output = #output_type;
            fn #fn_type -> #output_type {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn parse_variant(
    callable_type: CallableType,
    enum_name: &Ident,
    output_type: &TokenStream,
    variant: &Variant,
) -> Result<(TokenStream, TokenStream), syn::Error> {
    let variant_name = variant.ident.clone();

    let func_token = variant
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("argcall"));

    let trait_name = callable_type.as_trait();
    let fn_type = callable_type.as_fn();

    match &variant.fields {
        Fields::Unit => {
            let func_token = func_token
                .map(|attr| parse_fn_attribute(attr, std::iter::empty()))
                .next()
                .unwrap_or_else(|| {
                    Err(syn::Error::new_spanned(
                        variant,
                        "expected an 'argcall' attribute",
                    ))
                })?;

            let struct_name = Ident::new(
                &format!("{}{}Callable", enum_name, variant_name),
                variant_name.span(),
            );

            // Generate the struct for the variant
            let variant_struct = quote! {
                #[derive(Clone, Debug)]
                pub struct #struct_name;

                impl #trait_name for #struct_name {
                    type Output = #output_type;
                    fn #fn_type -> #output_type {
                        #func_token
                    }
                }
            };

            let match_arm = quote! {
                #enum_name::#variant_name => #func_token,
            };
            Ok((variant_struct, match_arm))
        }
        Fields::Unnamed(_) => {
            // like this:
            // #enum_name::#variant_name(value) => argcall::Callable::call_fn(value, ()),
            let match_arm = quote! {
                #enum_name::#variant_name(value) =>
            };

            let match_arm = match callable_type {
                CallableType::Callable => {
                    quote! { #match_arm argcall::Callable::call_fn(value, ()) }
                }
                CallableType::CallableMut => {
                    quote! { #match_arm argcall::CallableMut::call_fn_mut(value, ()) }
                }
                CallableType::CallableOnce => {
                    quote! { #match_arm argcall::CallableOnce::call_fn_once(value, ()) }
                }
            };
            Ok((TokenStream::new(), match_arm))
        }
        Fields::Named(fields) => {
            let names = fields
                .named
                .iter()
                .map(|field| field.ident.clone().unwrap());
            let func_token = func_token
                .map(|attr| parse_fn_attribute(attr, names.clone()))
                .next()
                .unwrap_or_else(|| {
                    Err(syn::Error::new_spanned(
                        variant,
                        "expected an 'argcall' attribute",
                    ))
                })?;

            let match_arm = quote! {
                #enum_name::#variant_name { #(#names),* } => #func_token,
            };
            Ok((TokenStream::new(), match_arm))
        }
    }
}

fn parse_output_attribute(attr: &Attribute) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut output = None;

    attr.parse_nested_meta(|meta| {
        let ident = meta.path.require_ident()?;
        if ident == "output" {
            let value = meta.value()?;
            output = Some(value.parse()?);
            return Ok(());
        }

        Err(meta.error(format!("unrecognized attribute for argcall: {}", ident)))
    })?;

    output.ok_or_else(|| syn::Error::new_spanned(attr, "expected an 'output' attribute"))
}

fn parse_fn_attribute(
    attr: &Attribute,
    args: impl Iterator<Item = Ident> + Clone,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut f = None;

    attr.parse_nested_meta(|meta| {
        let ident = meta.path.require_ident()?;
        if ident == "fn" {
            let value = meta.value()?;
            f = Some(value.parse()?);
            return Ok(());
        }
        if ident == "fn_path" {
            let value: LitStr = meta.value()?.parse()?;
            let ident = Ident::new(&value.value(), value.span());
            let args = args.clone();
            f = Some(quote! { #ident(#(#args),*) });
            return Ok(());
        }

        Err(meta.error(format!("unrecognized attribute for argcall: {}", ident)))
    })?;

    f.ok_or_else(|| syn::Error::new_spanned(attr, "expected an 'fn' attribute"))
}
