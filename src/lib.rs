#![recursion_limit = "256"]
// Coding conventions
#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    // unused_imports,
    dead_code,
    // missing_docs
)]

//! RGB21 library for working with fungible asset types, operating under
//! schemata, defined with LNPBP-21 standard:
//! - Root RGB21 schema, returned by [`schema::schema()`] with id
//!   [`SCHEMA_ID_BECH32`]
//! - RGB21 subschema, returned by [`schema::subschema()`], prohibiting asset
//!   replacement procedure and having id [`SUBSCHEMA_ID_BECH32`]
//! - High-level RGB21 API performing asset issuance, transfers and other
//!   asset-management operations

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;
#[macro_use]
extern crate rgb;
#[macro_use]
extern crate stens;

#[cfg(feature = "serde")]
extern crate serde_crate as serde;
#[cfg(feature = "serde")]
extern crate serde_with;

pub mod schema;
mod create;
mod asset;
mod transitions;

pub use asset::{Asset, Error};
pub use create::Rgb21;
pub use schema::{schema, subschema, SCHEMA_ID_BECH32, SUBSCHEMA_ID_BECH32};
