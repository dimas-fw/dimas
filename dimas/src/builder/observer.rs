// Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas_core::traits::Context;
// endregion:	--- modules

// region:		--- ObserverBuilder
/// The builder for an [`Observer`]
#[allow(clippy::module_name_repetitions)]
pub struct ObserverBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Context for the ObserverBuilder
	context: Context<P>,
}
// endregion:	--- ObserverBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<ObserverBuilder<Props>>();
	}
}
