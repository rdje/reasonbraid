//! Shared support for the node crate's integration tests.
//!
//! Each test binary compiles its own copy of this module and uses only part of
//! it, so an item one binary leaves unused would fail `-D warnings` there. The
//! allowance is scoped to this module; the tests themselves stay under the lint.
#![allow(dead_code)]

pub mod control_plane;
