use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Ident};

/// A procedural macro to derive the Callable trait
#[proc_macro_derive(Callable, attributes(argcall))]
pub fn callable_derive(input: TokenStream) -> TokenStream {
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

    // Process each variant in the enum
    for variant in data.variants {
        let variant_name = variant.ident;

        let func_path = variant
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("argcall"))
            .map(parse_fn_attribute)
            .next()
            .expect("Expected #[argcall(fn = \"...\")] attribute on variant")
            .unwrap();

        let struct_name = Ident::new(
            &format!("{}{}Callable", enum_name, variant_name),
            variant_name.span(),
        );

        // Generate the struct for the variant
        let variant_struct = quote! {
            #[derive(clap::Parser, Clone, Debug)]
            pub struct #struct_name;

            impl argcall::Callable for #struct_name {
                type Output = #output_type;
                fn call_fn(&self, _: ()) -> bool {
                    #func_path()
                }
            }
        };
        variant_structs.push(variant_struct);

        // Generate a match arm for the variant in the enum's `call_fn`
        let match_arm = quote! {
            #enum_name::#variant_name => #func_path(),
        };
        match_arms.push(match_arm);
    }

    let expanded = quote! {
        #(#variant_structs)*

        impl argcall::Callable for #enum_name {
            type Output = #output_type;
            fn call_fn(&self, _: ()) -> bool {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
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

fn parse_fn_attribute(attr: &Attribute) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut f = None;

    attr.parse_nested_meta(|meta| {
        let ident = meta.path.require_ident()?;
        if ident == "fn" {
            let value = meta.value()?;
            f = Some(value.parse()?);
            return Ok(());
        }

        Err(meta.error(format!("unrecognized attribute for argcall: {}", ident)))
    })?;

    f.ok_or_else(|| syn::Error::new_spanned(attr, "expected an 'fn' attribute"))
}
