#![cfg_attr(feature = "nightly", feature(proc_macro_tracked_path))]

use std::path::PathBuf;

use proc_macro::Span;
use syn::parse::Parse;
use syn::{Expr, ExprArray, LitStr, Token};
use quote::ToTokens;

use ppx_impl as ppx;

struct Args {
    file_path: String,
    base_path: String,
    params: Vec<String>,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let file_path: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let mut params: Option<ExprArray> = None;
        let base_path: LitStr = input.parse()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                params = Some(input.parse()?);
            }
        }

        let params = params
            .map(|params| {
                params.elems
                    .into_iter()
                    .map(|elem| match elem {
                        Expr::Lit(lit) => match &lit.lit {
                            syn::Lit::Str(s) => s.value(),
                            _ => panic!("Expected string literal in params"),
                        },
                        _ => panic!("Expected string literal in params")
                    }).collect::<Vec<_>>()
            })
            .unwrap_or(Vec::new());

        Ok(Args {
            file_path: file_path.value(),
            base_path: base_path.value(),
            params: params
        })
    }
}

/// Parse a macro at compile time from a file.
///
/// # Example
///
/// ```no_run
/// include_ppx_string!("path/to/file", "./templates", ["param1", "param2"])
/// ```
#[cfg(feature = "nightly")]
#[proc_macro]
pub fn include_ppx(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(input as Args);

    let source_path = PathBuf::from(Span::call_site().file());
    let base_path = source_path.parent().unwrap();

    let file_path = base_path.join(args.file_path);
    proc_macro::tracked::path(file_path.to_str().expect("File path was not UTF-8 encoded"));
    let base_path = base_path.join(args.base_path);

    let output = ppx::parse(file_path, base_path, args.params.iter().map(|s| s.as_str())).unwrap();
    let output = LitStr::new(&output, Span::call_site().into());

    return output.to_token_stream().into();
}

/// Parse a macro at compile time from a string.
///
/// # Example
///
/// ```rust
/// # use ppx_macros::include_ppx_string;
/// assert_eq!(
///     include_ppx_string!("#define A Hello\nA", ".", []),
///     "Hello"
/// );
/// ```
#[proc_macro]
pub fn include_ppx_string(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(input as Args);

    let source_path = PathBuf::from(Span::call_site().file());
    let base_path = source_path.parent().unwrap();

    let contents = args.file_path;
    let base_path = base_path.join(args.base_path);

    let output = ppx::parse_string(&contents, base_path, args.params.iter().map(|s| s.as_str())).unwrap();
    let output = LitStr::new(&output, Span::call_site().into());

    return output.to_token_stream().into();
}
