// Copyright © 2024 Stephan Kunz

//! Macro implementation
//!

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parser, punctuated::Punctuated, ItemFn, Meta, Token};

type Arguments = Punctuated<Meta, Token![,]>;

const UNSUPPORTED: &str = "not supported by macro";

struct Config {
	additional_threads: usize,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			additional_threads: 3usize,
		}
	}
}

fn parse_config(args: Arguments) -> Result<Config, syn::Error> {
	let mut config = Config::default();

	for arg in args {
		match arg {
			Meta::List(list) => {
				return Err(syn::Error::new_spanned(&list, UNSUPPORTED));
			}
			Meta::NameValue(named_value) => {
				// get ident
				let ident = named_value
					.path
					.get_ident()
					.ok_or_else(|| {
						syn::Error::new_spanned(&named_value, "must have a specified ident")
					})?
					.to_string()
					.to_lowercase();

				// check
				let lit = match &named_value.value {
					syn::Expr::Lit(syn::ExprLit { lit, .. }) => lit,
					expr => return Err(syn::Error::new_spanned(expr, "must be a literal")),
				};
				match ident.as_str() {
					"additional_threads" => {
						config.additional_threads = match lit {
							syn::Lit::Int(int_lit) => match int_lit.base10_parse::<usize>() {
								Ok(value) => value,
								Err(err) => {
									return Err(syn::Error::new(
										syn::spanned::Spanned::span(lit),
										format!("value `{ident}` is no positive integer: {err}"),
									))
								}
							},
							_ => {
								return Err(syn::Error::new(
									syn::spanned::Spanned::span(lit),
									format!("value `{ident}` is no positive integer"),
								))
							}
						};
					}
					_ => return Err(syn::Error::new_spanned(&named_value, UNSUPPORTED)),
				}
			}
			Meta::Path(path) => {
				return Err(syn::Error::new_spanned(&path, UNSUPPORTED));
			}
		}
	}

	Ok(config)
}

pub fn main(args: TokenStream, main_fn: TokenStream) -> TokenStream {
	// save original for creation of result with error
	let mut result_with_error = main_fn.clone();

	// parse the `main()` function
	let mut main_fn: ItemFn = match syn::parse2(main_fn) {
		Ok(item) => item,
		Err(error) => {
			result_with_error.extend(error.into_compile_error());
			return result_with_error;
		}
	};

	// check given function beeing a proper `asyn main()` function
	if main_fn.sig.ident != "main" {
		let err = syn::Error::new_spanned(
			&main_fn.sig.ident,
			"macro can only be used for main function",
		);
		result_with_error.extend(err.into_compile_error());
		return result_with_error;
	}
	if !main_fn.sig.inputs.is_empty() {
		let err = syn::Error::new_spanned(
			&main_fn.sig.ident,
			"the main function cannot accept arguments",
		);
		result_with_error.extend(err.into_compile_error());
		return result_with_error;
	}
	if main_fn.sig.asyncness.is_none() {
		let err = syn::Error::new_spanned(
			main_fn.sig.fn_token,
			"missing `async` keyword in function declaration",
		);
		result_with_error.extend(err.into_compile_error());
		return result_with_error;
	}

	// parse args
	let config = Arguments::parse_terminated
		.parse2(args)
		.and_then(parse_config);

	match config {
		Ok(config) => {
			// remove `async` from function signature
			main_fn.sig.asyncness = None;

			// variables for quote macro
			let num_threads = config.additional_threads;
			let signature = main_fn.sig;
			let body = main_fn.block;

			// possible extensions see: https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html#
			quote! {
				#signature {
					tokio::runtime::Builder::new_multi_thread()
						.enable_all()
						.worker_threads(#num_threads)
						.thread_name_fn(|| "dimas_worker".into())
						.build()
						.unwrap()
						.block_on(async #body)
				}
			}
		}
		Err(err) => {
			result_with_error.extend(err.into_compile_error());
			result_with_error
		}
	}
}
