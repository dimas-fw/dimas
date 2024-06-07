// Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas_core::traits::Context;
// endregion:	--- modules

// region:		--- Observer
/// Observer
pub struct Observer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Context for the Observer
	context: Context<P>,
}
// endregion:	--- Observer

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Observer<Props>>();
	}
}
