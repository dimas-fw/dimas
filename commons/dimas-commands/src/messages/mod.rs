// Copyright © 2024 Stephan Kunz
#![allow(clippy::non_canonical_partial_ord_impl)]

//! Module `Messages` provides the different messages used with DiMAS.

mod about_entity;
mod ping_entity;
mod scouting_entity;

// flatten
pub use about_entity::*;
pub use ping_entity::*;
pub use scouting_entity::*;
