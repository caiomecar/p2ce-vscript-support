use proc_macro::TokenStream;
use quote::quote;
use std::path::PathBuf;
use syn::{LitStr, parse_macro_input};

fn read_file(rel_path: &str) -> (String, Vec<String>) {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let abs_path: PathBuf = PathBuf::from(&manifest_dir).join(rel_path);
    let path_str = abs_path.to_str().expect("non-UTF-8 path").to_string();
    let src = std::fs::read_to_string(&abs_path)
        .unwrap_or_else(|e| panic!("cannot read `{}`: {}", abs_path.display(), e));
    let entries: Vec<String> = src
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();
    assert!(
        !entries.is_empty(),
        "`{}` contains no entries",
        abs_path.display()
    );
    (path_str, entries)
}

fn build_set(input: TokenStream, case_insensitive: bool) -> TokenStream {
    let rel_path = parse_macro_input!(input as LitStr).value();
    let (path_str, entries) = read_file(&rel_path);

    let keys: Vec<String> = entries
        .iter()
        .map(|e| {
            if case_insensitive {
                e.to_lowercase()
            } else {
                e.clone()
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

fn build_map(input: TokenStream, case_insensitive: bool) -> TokenStream {
    let rel_path = parse_macro_input!(input as LitStr).value();
    let (path_str, raw_entries) = read_file(&rel_path);

    let entries: Vec<(String, String)> = raw_entries
        .iter()
        .map(|e| {
            let mut parts = e.splitn(2, '$');
            let key = parts
                .next()
                .unwrap_or_else(|| panic!("`{e}` is missing key"));
            let key = if case_insensitive {
                key.to_lowercase()
            } else {
                key.to_string()
            };

            let value = format!(
                "{:?}",
                parts
                    .next()
                    .unwrap_or_else(|| panic!("`{e}` is missing value"))
            );

            (key, value)
        })
        .collect();

    let mut builder = phf_codegen::Map::new();
    for (key, value) in entries {
        builder.entry(key, value);
    }
    let map_code = builder.build().to_string();
    let map_tokens: proc_macro2::TokenStream = map_code
        .parse()
        .expect("phf_codegen produced invalid tokens");

    let expanded = quote! {
        {
            const _: &str = include_str!(#path_str);
            #map_tokens
        }
    };
    expanded.into()
}

#[proc_macro]
/// # Panics
pub fn case_insensitive_set(input: TokenStream) -> TokenStream {
    build_set(input, true)
}

#[proc_macro]
/// # Panics
pub fn case_sensitive_set(input: TokenStream) -> TokenStream {
    build_set(input, false)
}

#[proc_macro]
/// # Panics
pub fn case_insensitive_map(input: TokenStream) -> TokenStream {
    build_map(input, true)
}

#[proc_macro]
/// # Panics
pub fn case_sensitive_map(input: TokenStream) -> TokenStream {
    build_map(input, false)
}
