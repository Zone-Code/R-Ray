use std::{env, fs, io, path::PathBuf};

use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, LitStr,
};

macro_rules! unwrap_or_ce {
    (
        $result:expr,
        $span:expr,
        $err:ident => $msg:expr$(,)?
    ) => {
        match $result {
            Ok(ok) => ok,
            Err($err) => return syn::Error::new($span, $msg).to_compile_error().into(),
        }
    };
    (
        $result:expr,
        $span:expr$(,)?
    ) => {
        match $result {
            Ok(ok) => ok,
            Err(err) => return syn::Error::new($span, err).to_compile_error().into(),
        }
    };
}

/// Generates declarations of directory's icons' paths
///
/// # Arguments
///
/// * `input`: path to your icons' directory.
#[proc_macro]
#[cfg(target_family = "windows")]
pub fn typesafe_icons(input: TokenStream) -> TokenStream {
    let Args { path_to_dir } = parse_macro_input!(input as Args);

    let manifest_dir = unwrap_or_ce!(env::var("CARGO_MANIFEST_DIR"), Span::call_site(),);

    let path: PathBuf = path_to_dir.value().into();
    let path = unwrap_or_ce!(
        PathBuf::from(&manifest_dir).join(path).canonicalize(),
        path_to_dir.span(),
        err => format!("{err}\nRoot path: {}", manifest_dir),
    );

    let icon_paths = unwrap_or_ce!(
        process_path(path),
        path_to_dir.span(),
        err => format!("{err}\nRoot path: {}", manifest_dir),
    );

    let tokens = icon_paths.into_iter().map(|path| {
        let name = path.file_stem().unwrap();
        let name = name
            .to_string_lossy()
            .replace(|ch: char| !ch.is_ascii_alphanumeric(), "_");

        let name_upper = Ident::new(&name.to_ascii_uppercase(), Span::call_site());
        let name_lower = Ident::new(format!("icon_{}", name.to_ascii_lowercase()).as_str(), Span::call_site());
        let path = &path.to_string_lossy()[4..]; // це у блок

        #[cfg(target_family = "windows")]
        {
            
        }

        quote! {
            pub const #name_upper: &'static str = #path;
            #[macro_export]
            macro_rules! #name_lower {
                () => { #path };
            }
            pub use #name_lower;
        }
    });

    quote! {
        #(#tokens)*
    }
    .into()
}

fn process_path(path: PathBuf) -> io::Result<Vec<PathBuf>> {
    let read_dir = fs::read_dir(path)?;

    let mut files: Vec<PathBuf> = vec![];

    itertools::process_results(read_dir, |iter| -> io::Result<()> {
        for entry in iter {
            if !entry.metadata()?.is_file() {
                continue;
            }
            files.push(entry.path().canonicalize()?)
        }

        Ok(())
    })??;

    Ok(files)
}

struct Args {
    path_to_dir: LitStr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path_to_dir: input.parse()?
        })
    }
}
