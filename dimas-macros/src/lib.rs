// Copyright © 2024 Stephan Kunz

//! `#[main(...)]` macro for `DiMAS`
//!

extern crate proc_macro;

mod r#impl;

use proc_macro::TokenStream;

/// Marks async main functions to be executed by a multi threaded tokio runtime
///
/// Note: This macro can only be used on the `main` function.
/// 
/// # Usage
/// ```no_test
/// #[dimas::main]
/// async fn main() {
///     // your code
///     ...
/// }
/// ```
/// 
/// ## Increase Worker threads
/// `DiMAS` creates a minimum of 3 worker threads within tokio runtime.
/// 
/// To increase the amount of worker threads, the macro can be configured using
///
/// ```no_test
/// #[dimas::main(additional_threads = 5)]  // adds additional 5 threads to the default of 3
/// ```
/// 
#[proc_macro_attribute]
pub fn main(metadata: TokenStream, input: TokenStream) -> TokenStream {
	// call implementation with conversion to and from proc-macro2 library
	r#impl::main(metadata.into(), input.into()).into()
}
