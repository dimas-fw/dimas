//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use super::communicator::Communicator;
use crate::prelude::*;
use std::sync::{Arc, RwLock};
// endregion:	--- modules

// region:		--- types
//#[allow(clippy::module_name_repetitions)]
//pub type PublisherCallback<P> = fn(Arc<Context>, Arc<RwLock<P>>, sample: Sample);
// endregion:	--- types

// region:		--- PublisherBuilder
#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone)]
pub struct PublisherBuilder<'a, P> {
	pub(crate) collection: Arc<RwLock<Vec<Publisher<'a>>>>,
	pub(crate) communicator: Arc<Communicator>,
	pub(crate) props: Arc<RwLock<P>>,
	pub(crate) key_expr: Option<String>,
	pub(crate) msg_type: Option<String>,
}

impl<'a, P> PublisherBuilder<'a, P> 
where
	P: Send + Sync + Unpin + 'static,
{
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	pub async fn add(mut self) -> Result<()> {
		if self.key_expr.is_none() && self.msg_type.is_none() {
			return Err("No key expression or msg type given".into());
		}

		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().expect("should never happen")
		} else {
			self.communicator.clone().prefix()
				+ "/" + &self.msg_type.expect("should never happen")
				+ "/*"
		};

		//dbg!(&key_expr);
		let _props = self.props.clone();
		let publ = self.communicator.create_publisher(key_expr).await;
		let p = Publisher { _publisher: publ };
		self.collection
			.write()
			.expect("should never happen")
			.push(p);
		Ok(())
	}
}
// endregion:	--- PublisherBuilder

// region:		--- Publisher
pub struct Publisher<'a> {
	_publisher: zenoh::publication::Publisher<'a>,
}
// endregion:	--- Publisher

#[cfg(test)]
mod tests {
	use super::*;

	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Publisher>();
		is_normal::<PublisherBuilder<Props>>();
	}
}
