// Copyright © 2024 Stephan Kunz
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(clippy::manual_assert)]

//! Macros for `DiMAS`
//!

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet as Set;
use syn::{
	parse::{Parse, ParseStream, Result},
	parse_macro_input,
	punctuated::Punctuated,
	Ident, ItemFn, Token,
};

struct Args {
	vars: Set<Ident>,
}

impl Parse for Args {
	fn parse(input: ParseStream) -> Result<Self> {
		let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
		Ok(Self {
			vars: vars.into_iter().collect(),
		})
	}
}

/// main wrapper
///
/// # Panics
/// - if a main function has parameters/attributes
#[proc_macro_attribute]
pub fn main(metadata: TokenStream, input: TokenStream) -> TokenStream {
	// parse the arguments given in metadata
	let args = parse_macro_input!(metadata as Args);

	// parse the rust code given by input
	let mut input_fn = parse_macro_input!(input as ItemFn);
	if input_fn.sig.ident != "main" {
		panic!("can only be used for main function");
	}
	if !input_fn.sig.inputs.is_empty() {
		panic!("the main function cannot accept arguments");
	}
	// remove asyncness from function signature
	input_fn.sig.asyncness = None;
	let signature = input_fn.sig;
	let body = input_fn.block; //.stmts;

	// possible extensions see: https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html#
	TokenStream::from(quote! {
		#signature {
			tokio::runtime::Builder::new_multi_thread()
				.enable_all()
				.worker_threads(3)
				.thread_name_fn(|| "dimas_worker".into())
				.build()
				.unwrap()
				.block_on(async #body)
		}
	})
}
