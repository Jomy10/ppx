#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "nightly", feature(proc_macro_tracked_path))]

use std::path::PathBuf;

use proc_macro::Span;
use quote::quote;
use syn::parse::Parse;
use syn::{Expr, ExprArray, LitStr, Token};

struct Args {
    contents_or_path: String,
    base_path: String,
    params: Vec<String>,
    label: Option<String>,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let contents_or_path: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let mut params: Option<ExprArray> = None;
        let base_path: LitStr = input.parse()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                params = Some(input.parse()?);
            }
        }

        let mut label: Option<LitStr> = None;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                label = Some(input.parse()?);
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

        if !input.is_empty() {
            if input.peek(Token![,]) {
                _ = input.parse::<Token![,]>()?;
                if !input.is_empty() {
                    panic!("Unexpected token(s) in input {:?}", input);
                }
            } else {
                panic!("Unexpected token(s) in input {:?}", input);
            }
        }

        Ok(Args {
            contents_or_path: contents_or_path.value(),
            base_path: base_path.value(),
            params: params,
            label: label.map(|l| l.value())
        })
    }
}

#[cfg(feature = "nightly")]
#[proc_macro]
pub fn include_wgsl_template(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(input as Args);

    let source_path = PathBuf::from(Span::call_site().file());
    let base_path = source_path.parent().unwrap();

    let file_path = base_path.join(args.contents_or_path);
    proc_macro::tracked::path(file_path.to_str().expect("File path was not UTF-8 encoded"));
    let base_path = base_path.join(args.base_path);

    let output = match ppx::parse(&file_path, base_path, args.params.iter().map(|s| s.as_str())) {
        Ok(out) => out,
        Err(err) => panic!("{}", err),
    };
    let output = LitStr::new(&output, Span::call_site().into());

    let label = LitStr::new(&args.label.unwrap_or(file_path.to_str().unwrap().to_string()), Span::call_site().into());

    return quote! {
        ::wgpu::ShaderModuleDescriptor {
            label: Some(#label),
            source: ::wgpu::ShaderSource::Wgsl(#output.into()),
        }
    }.into();
}

#[proc_macro]
pub fn include_wgsl_template_string(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(input as Args);

    let source_path = PathBuf::from(Span::call_site().file());
    let base_path = source_path.parent().unwrap();

    let contents = args.contents_or_path;
    let base_path = base_path.join(args.base_path);

    let output = ppx::parse_string(&contents, base_path, args.params.iter().map(|s| s.as_str())).unwrap();
    let output = LitStr::new(&output, Span::call_site().into());

    let label = LitStr::new(&args.label.unwrap_or(contents[0..std::cmp::min(contents.len(), 25)].to_string()), Span::call_site().into());

    return quote! {
        ::wgpu::ShaderModuleDescriptor {
            label: Some(#label),
            source: ::wgpu::ShaderSource::Wgsl(#output.into()),
        }
    }.into();
}
