// Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas_core::traits::Context;
// endregion:	--- modules

// region:		--- ObservableBuilder
/// The builder for an [`Observable`]
#[allow(clippy::module_name_repetitions)]
pub struct ObservableBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Context for the ObservableBuilder
	context: Context<P>,
}
// endregion:	--- ObservableBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<ObservableBuilder<Props>>();
	}
}
