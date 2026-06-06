//! UI components and utilities module.
//!
//! Re-exports the `h_flex`/`v_flex` free functions, the `StyledExt` trait,
//! and all UI component modules.

pub mod common;
pub mod components;
pub mod pages;
pub(crate) mod theme;

pub use common::ext::StyledExt;
pub use common::stack::{h_flex, v_flex};
