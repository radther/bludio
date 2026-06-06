//! UI components and utilities module.
//!
//! Re-exports the `h_flex`/`v_flex` free functions, the `StyledExt` trait,
//! and all UI component modules.

pub mod accessibility;
pub mod animation;
pub mod audio;
pub mod bluetooth;
pub mod components;
pub mod dev_test_page;
pub mod ext;
pub mod icons;
pub mod settings_page;
pub mod stack;
pub mod tab_bar;
pub(crate) mod theme;
pub mod tooltip;

pub use ext::StyledExt;
pub use stack::{h_flex, v_flex};
