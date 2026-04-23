use proc_macro::TokenStream;
use quote::quote;
use std::path::PathBuf;
use syn::{LitStr, parse_macro_input};

fn build_set(input: TokenStream, case_insensitive: bool) -> TokenStream {
    let rel_path = parse_macro_input!(input as LitStr).value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let abs_path: PathBuf = PathBuf::from(&manifest_dir).join(&rel_path);
    let path_str = abs_path.to_str().expect("non-UTF-8 path");

    let src = std::fs::read_to_string(&abs_path)
        .unwrap_or_else(|e| panic!("cannot read `{}`: {}", abs_path.display(), e));

    let entries: Vec<&str> = src
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    assert!(
        !entries.is_empty(),
        "`{}` contains no entries",
        abs_path.display()
    );

    let keys: Vec<String> = entries
        .iter()
        .map(|e| {
            if case_insensitive {
                e.to_lowercase()
            } else {
                String::from(*e)
            }
        })
        .collect();

    let mut builder = phf_codegen::Set::new();
    for key in &keys {
        builder.entry(key.as_str());
    }
    let set_code = builder.build().to_string();
    let set_tokens: proc_macro2::TokenStream = set_code
        .parse()
        .expect("phf_codegen produced invalid tokens");

    let completion_literals = entries.iter().map(|e| quote! { #e });

    let expanded = quote! {
        {
            const _: &str = include_str!(#path_str);

            (
                #set_tokens,

                &[ #(#completion_literals),* ] as &[&str],
            )
        }
    };

    expanded.into()
}

#[proc_macro]
pub fn case_insensetive_set(input: TokenStream) -> TokenStream {
    build_set(input, true)
}

#[proc_macro]
pub fn case_sensetive_set(input: TokenStream) -> TokenStream {
    build_set(input, false)
}
