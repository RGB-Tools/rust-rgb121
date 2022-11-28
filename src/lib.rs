#![recursion_limit = "256"]
// Coding conventions
#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_imports,
    dead_code,
    missing_docs
)]

//! RGB121 library for working with fungible asset types, operating under
//! schemata, defined with LNPBP-121 standard:
//! - Root RGB121 schema, returned by [`schema::schema()`] with id
//!   [`SCHEMA_ID_BECH32`]
//! - RGB121 subschema, returned by [`schema::subschema()`], prohibiting asset
//!   replacement procedure and having id [`SUBSCHEMA_ID_BECH32`]
//! - High-level RGB121 API performing asset issuance, transfers and other
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

mod schema;
mod create;
mod asset;
mod transitions;

pub use asset::{Asset, Error};
pub use create::{Error as CreateError, FileAttachment, Rgb121};
pub use schema::{
    schema, subschema, FieldType, OwnedRightType, SCHEMA_ID_BECH32, SUBSCHEMA_ID_BECH32,
};
