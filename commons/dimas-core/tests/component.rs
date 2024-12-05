//! Copyright © 2024 Stephan Kunz

use dimas_core::{
	Activity, ActivityId, Component, ComponentId, ComponentType, OperationState, Operational,
	OperationalType, Transitions,
};
use std::fmt::Debug;

#[dimas_macros::component]
#[derive(Debug)]
struct TestComponent1<P>
where
	P: Debug + Send + Sync,
{
	dummy: P,
}

impl<P> Transitions for TestComponent1<P> where P: Debug + Send + Sync {}

#[dimas_macros::component]
#[derive(Debug, Default)]
struct TestComponent {}

impl TestComponent {}

impl Transitions for TestComponent {}

#[test]
fn component() {
	let mut component = TestComponent::default();
	assert_eq!(component.id(), "");
	component.set_id("new id".into());
	assert_eq!(component.id(), "new id");
}
