use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, LitStr, Variant};

/// A procedural macro to derive the Callable trait
#[proc_macro_derive(Callable, attributes(argcall))]
pub fn callable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input token stream as a DeriveInput struct
    let input = parse_macro_input!(input as DeriveInput);

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
            let (variant_struct, match_arm) = parse_variant(&enum_name, &output_type, variant)?;
            variant_structs.push(variant_struct);
            match_arms.push(match_arm);
            Ok::<(), syn::Error>(())
        })
        .unwrap();

    let expanded = quote! {
        #(#variant_structs)*

        impl argcall::Callable for #enum_name {
            type Output = #output_type;
            fn call_fn(&self, _: ()) -> #output_type {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn parse_variant(
    enum_name: &Ident,
    output_type: &TokenStream,
    variant: &Variant,
) -> Result<(TokenStream, TokenStream), syn::Error> {
    let variant_name = variant.ident.clone();

    let func_token = variant
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("argcall"));

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

                impl argcall::Callable for #struct_name {
                    type Output = #output_type;
                    fn call_fn(&self, _: ()) -> #output_type {
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
            let match_arm = quote! {
                #enum_name::#variant_name(value) => argcall::Callable::call_fn(value, ()),
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
