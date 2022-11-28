//! RGB121 schemata defining fungible asset smart contract prototypes.

use std::str::FromStr;

use rgb::schema::{
    DiscreteFiniteFieldFormat, GenesisSchema, Occurrences, Schema, SchemaId, StateSchema,
    TransitionSchema,
};
use rgb::script::OverrideRules;
use rgb::vm::embedded::constants::*;
use rgb::ValidationScript;
use stens::{PrimitiveType, StructField, TypeRef, TypeSystem};

/// Schema identifier for full RGB121 fungible asset
pub const SCHEMA_ID_BECH32: &str =
    "rgbsh1ykclt9qxkskqt88dwgccsp4w624k7adjwj06sknjkh04ygtc7rqsnykld7";

/// Schema identifier for full RGB121 fungible asset subschema prohibiting
/// engraving operation
pub const SUBSCHEMA_ID_BECH32: &str =
    "rgbsh1ep4k4qvghntwptcn0gqmfpdvr8vz3amslvy4pc7s32u7h500l5hqxlzjsk";

/// Field types for RGB121 schemata
///
/// Subset of known RGB schema pre-defined types applicable to fungible assets.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(Debug)]
#[repr(u16)]
pub enum FieldType {
    /// Asset name
    ///
    /// Used within context of genesis
    Name = FIELD_TYPE_NAME,

    /// Text of the asset contract
    ///
    /// Used within context of genesis
    RicardianContract = FIELD_TYPE_CONTRACT_TEXT,

    /// Asset description
    ///
    /// Used within context of genesis
    Description = FIELD_TYPE_COMMENTARY,

    /// Asset data
    Data = FIELD_TYPE_DATA,

    /// Asset data format
    DataFormat = FIELD_TYPE_DATA_FORMAT,

    /// Decimal precision
    Precision = FIELD_TYPE_PRECISION,

    /// Supply issued with the genesis
    IssuedSupply = FIELD_TYPE_ISSUED_SUPPLY,

    /// Timestamp for genesis
    Timestamp = FIELD_TYPE_TIMESTAMP,

    /// Asset parent ID
    ParentId = FIELD_TYPE_PARENT_ID,
}

impl From<FieldType> for rgb::schema::FieldType {
    #[inline]
    fn from(ft: FieldType) -> Self {
        ft as rgb::schema::FieldType
    }
}

/// Owned right types used by RGB121 schemata
///
/// Subset of known RGB schema pre-defined types applicable to fungible assets.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(Debug)]
#[repr(u16)]
pub enum OwnedRightType {
    /// Asset ownership right
    Assets = STATE_TYPE_OWNERSHIP_RIGHT,

    /// Asset engraving right
    Engraving = STATE_TYPE_OWNERSHIP_RIGHT + 1,
}

impl From<OwnedRightType> for rgb::schema::OwnedRightType {
    #[inline]
    fn from(t: OwnedRightType) -> Self {
        t as rgb::schema::OwnedRightType
    }
}

/// State transition types defined by RGB121 schemata
///
/// Subset of known RGB schema pre-defined types applicable to fungible assets.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(Debug)]
#[repr(u16)]
pub enum TransitionType {
    /// Secondary issuance
    Issue = TRANSITION_TYPE_ISSUE_NFT,

    /// Asset transfer
    Transfer = TRANSITION_TYPE_VALUE_TRANSFER,

    /// Asset engraving
    Engraving = TRANSITION_TYPE_ENGRAVING,
}

impl From<TransitionType> for rgb::schema::TransitionType {
    #[inline]
    fn from(t: TransitionType) -> Self {
        t as rgb::schema::TransitionType
    }
}

fn type_system() -> TypeSystem {
    type_system! {
        "OutPoint" :: {
            StructField::with("Txid"),
            StructField::primitive(PrimitiveType::U16),
        },
        "Txid" :: { StructField::array(PrimitiveType::U8, 32) }
    }
}

fn genesis() -> GenesisSchema {
    use Occurrences::*;

    GenesisSchema {
        metadata: type_map! {
            FieldType::Name => Once,
            FieldType::RicardianContract => NoneOrOnce,
            FieldType::Description => NoneOrOnce,
            FieldType::Data => NoneOrMore,
            FieldType::DataFormat => NoneOrOnce,
            FieldType::Precision => Once,
            FieldType::Timestamp => Once,
            FieldType::IssuedSupply => Once,
            FieldType::ParentId => NoneOrOnce
        },
        owned_rights: type_map! {
            OwnedRightType::Assets => NoneOrMore,
            OwnedRightType::Engraving => NoneOrMore
        },
        public_rights: none!(),
    }
}

fn issue() -> TransitionSchema {
    use Occurrences::*;

    TransitionSchema {
        metadata: type_map! {
            // We need this field in order to be able to verify pedersen
            // commitments
            FieldType::IssuedSupply => Once,
            FieldType::Data => NoneOrMore,
            FieldType::DataFormat => NoneOrOnce
        },
        closes: none!(),
        owned_rights: type_map! {
            OwnedRightType::Assets => NoneOrMore,
            OwnedRightType::Engraving => NoneOrMore
        },
        public_rights: none!(),
    }
}

/// Builds & returns complete RGB121 schema (root schema object)
pub fn schema() -> Schema {
    use Occurrences::*;

    Schema {
        rgb_features: none!(),
        root_id: none!(),
        genesis: genesis(),
        type_system: type_system(),
        extensions: none!(),
        transitions: type_map! {
            TransitionType::Issue => issue(),
            TransitionType::Transfer => TransitionSchema {
                metadata: none!(),
                closes: type_map! {
                    OwnedRightType::Assets => OnceOrMore
                },
                owned_rights: type_map! {
                    OwnedRightType::Assets => NoneOrMore
                },
                public_rights: none!()
            },
            TransitionType::Engraving => TransitionSchema {
                metadata: type_map! {
                    FieldType::Data => NoneOrMore,
                    FieldType::DataFormat => NoneOrOnce
                },
                closes: type_map! {
                    OwnedRightType::Assets => OnceOrMore,
                    OwnedRightType::Engraving => NoneOrMore
                },
                owned_rights: type_map! {
                    OwnedRightType::Assets => NoneOrMore,
                    OwnedRightType::Engraving => NoneOrMore
                },
                public_rights: none!()
            }
        },
        field_types: type_map! {
            // Rational: if we will use just 26 letters of English alphabet (and
            // we are not limited by them), we will have 26^8 possible tickers,
            // i.e. > 208 trillions, which is sufficient amount
            FieldType::Name => TypeRef::ascii_string(),
            FieldType::RicardianContract => TypeRef::ascii_string(),
            FieldType::Description => TypeRef::ascii_string(),
            FieldType::Data => TypeRef::bytes(),
            FieldType::DataFormat => TypeRef::u16(),
            // Contract text may contain URL, text or text representation of
            // Ricardian contract, up to 64kb. If the contract doesn't fit, a
            // double SHA256 hash and URL should be used instead, pointing to
            // the full contract text, where hash must be represented by a
            // hexadecimal string, optionally followed by `\n` and text URL
            FieldType::Precision => TypeRef::u8(),
            // We need this b/c allocated amounts are hidden behind Pedersen
            // commitments
            FieldType::IssuedSupply => TypeRef::u64(),
            // While UNIX timestamps allow negative numbers; in context of RGB
            // Schema, assets can't be issued in the past before RGB or Bitcoin
            // even existed; so we prohibit all the dates before RGB release
            // This timestamp is equal to 10/10/2020 @ 2:37pm (UTC)
            FieldType::Timestamp => TypeRef::i64(),
            FieldType::ParentId => TypeRef::ascii_string()
        },
        owned_right_types: type_map! {
            // How much issuer can issue tokens on this path. If there is no
            // limit, than `core::u64::MAX` / sum(inflation_assignments)
            // must be used, as this will be a de-facto limit to the
            // issuance
            OwnedRightType::Assets => StateSchema::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit),
            OwnedRightType::Engraving => StateSchema::DataContainer
        },
        public_right_types: none!(),
        script: ValidationScript::Embedded,
        override_rules: OverrideRules::AllowAnyVm,
    }
}

/// RGB121 subschema which allows simple asset transfers but no engraving
pub fn subschema() -> Schema {
    use Occurrences::*;

    Schema {
        rgb_features: none!(),
        root_id: SchemaId::from_str(SCHEMA_ID_BECH32)
            .expect("Broken root schema ID for RGB121 sub-schema"),
        type_system: type_system(),
        genesis: genesis(),
        extensions: none!(),
        transitions: type_map! {
            TransitionType::Issue => issue(),
            TransitionType::Transfer => TransitionSchema {
                metadata: none!(),
                closes: type_map! {
                    OwnedRightType::Assets => OnceOrMore
                },
                owned_rights: type_map! {
                    OwnedRightType::Assets => NoneOrMore
                },
                public_rights: none!()
            }
        },
        field_types: type_map! {
            // Rational: if we will use just 26 letters of English alphabet (and
            // we are not limited by them), we will have 26^8 possible tickers,
            // i.e. > 208 trillions, which is sufficient amount
            FieldType::Name => TypeRef::ascii_string(),
            FieldType::RicardianContract => TypeRef::ascii_string(),
            FieldType::Description => TypeRef::ascii_string(),
            FieldType::Data => TypeRef::bytes(),
            FieldType::DataFormat => TypeRef::u16(),
            FieldType::Precision => TypeRef::u8(),
            // We need this b/c allocated amounts are hidden behind Pedersen
            // commitments
            FieldType::IssuedSupply => TypeRef::u64(),
            // While UNIX timestamps allow negative numbers; in context of RGB
            // Schema, assets can't be issued in the past before RGB or Bitcoin
            // even existed; so we prohibit all the dates before RGB release
            // This timestamp is equal to 10/10/2020 @ 2:37pm (UTC)
            FieldType::Timestamp => TypeRef::i64(),
            FieldType::ParentId => TypeRef::ascii_string()
        },
        owned_right_types: type_map! {
            OwnedRightType::Assets => StateSchema::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit),
            OwnedRightType::Engraving => StateSchema::DataContainer
        },
        public_right_types: none!(),
        script: ValidationScript::Embedded,
        override_rules: OverrideRules::AllowAnyVm,
    }
}

#[cfg(test)]
mod test {
    use lnpbp::bech32::Bech32ZipString;
    use rgb::schema::SchemaVerify;
    use rgb::Validity;
    use strict_encoding::{StrictDecode, StrictEncode};

    use super::*;

    #[test]
    fn schema_id() {
        let id = schema().schema_id();
        assert_eq!(id.to_string(), SCHEMA_ID_BECH32);
        assert_eq!(
            id.to_string(),
            "rgbsh1ykclt9qxkskqt88dwgccsp4w624k7adjwj06sknjkh04ygtc7rqsnykld7"
        );
    }

    #[test]
    fn subschema_id() {
        let id = subschema().schema_id();
        assert_eq!(id.to_string(), SUBSCHEMA_ID_BECH32);
        assert_eq!(
            id.to_string(),
            "rgbsh1ep4k4qvghntwptcn0gqmfpdvr8vz3amslvy4pc7s32u7h500l5hqxlzjsk"
        );
    }

    #[test]
    fn schema_strict_encode() {
        let data = schema()
            .strict_serialize()
            .expect("RGB-121 schema serialization failed");

        let bech32data = data.bech32_zip_string();
        println!("{}", bech32data);

        let schema121 =
            Schema::strict_deserialize(data).expect("RGB-121 schema deserialization failed");

        assert_eq!(schema(), schema121);
        assert_eq!(format!("{:#?}", schema()), format!("{:#?}", schema121));
        assert_eq!(
            bech32data,
            "z1qxz56wgwcfqqe894xuf8fzlcqqlc9q386zjgzfjg8sn5fapuytlkyxtmjv3g0994wej8k0pnmp7gy8wweln0\
            928035szpch64fhjf33kqr5ug7w0p58r7v2s2dr0ucm9vv3tp2rfp8df0wehq5wmg29dc3wat7lgtrucvuhya3a\
            tf0h0h5mskce6g79lghpjv6whxxrcuzfkfc3zcdz4rnnahey4wv43ejfn7eajpk0"
        );
    }

    #[test]
    fn subschema_verify() {
        let status = subschema().schema_verify(&schema());
        assert_eq!(status.validity(), Validity::Valid);
    }
}
