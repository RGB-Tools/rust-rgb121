use std::collections::BTreeMap;

use bitcoin::OutPoint;
use chrono::Utc;
use lnpbp::chain::Chain;
use rgb::fungible::allocation::{
    AllocationMap, IntoSealValueMap, OutpointValueMap, OutpointValueVec,
};
use rgb::{
    data, secp256k1zkp, value, Assignment, Consignment, Contract, Genesis, TypedAssignments,
};
use stens::AsciiString;

use crate::schema;
use crate::schema::{FieldType, OwnedRightType};

/// Extension trait for consignments defining RGB21-specific API.
#[allow(clippy::too_many_arguments)]
pub trait Rgb21<'consignment>: Consignment<'consignment> {
    /// Performs primary asset issue, producing [`Contract`] consignment.
    fn create_rgb21(
        chain: Chain,
        ticker: AsciiString,
        name: AsciiString,
        precision: u8,
        allocations: OutpointValueVec,
        inflation: OutpointValueMap,
        renomination: Option<OutPoint>,
        epoch: Option<OutPoint>,
    ) -> Contract;
}

impl<'consignment> Rgb21<'consignment> for Contract {
    fn create_rgb21(
        chain: Chain,
        ticker: AsciiString,
        name: AsciiString,
        precision: u8,
        allocations: OutpointValueVec,
        inflation: OutpointValueMap,
        renomination: Option<OutPoint>,
        epoch: Option<OutPoint>,
    ) -> Contract {
        let now = Utc::now().timestamp();
        let mut metadata = type_map! {
            FieldType::Ticker => field!(AsciiString, ticker),
            FieldType::Name => field!(AsciiString, name),
            FieldType::Precision => field!(U8, precision),
            FieldType::Timestamp => field!(I64, now)
        };

        let issued_supply = allocations.sum();
        let mut owned_rights = BTreeMap::new();
        owned_rights.insert(
            OwnedRightType::Assets.into(),
            TypedAssignments::zero_balanced(
                vec![value::Revealed {
                    value: issued_supply,
                    blinding: secp256k1zkp::key::ONE_KEY.into(),
                }],
                allocations.into_seal_value_map(),
                empty![],
            ),
        );
        metadata.insert(FieldType::IssuedSupply.into(), field!(U64, issued_supply));

        if !inflation.is_empty() {
            owned_rights.insert(
                OwnedRightType::Inflation.into(),
                inflation.into_assignments(),
            );
        }

        if let Some(outpoint) = renomination {
            owned_rights.insert(
                OwnedRightType::Renomination.into(),
                TypedAssignments::Void(vec![Assignment::Revealed {
                    seal: outpoint.into(),
                    state: data::Void(),
                }]),
            );
        }

        if let Some(outpoint) = epoch {
            owned_rights.insert(
                OwnedRightType::BurnReplace.into(),
                TypedAssignments::Void(vec![Assignment::Revealed {
                    seal: outpoint.into(),
                    state: data::Void(),
                }]),
            );
        }

        let schema = schema::schema();

        let genesis = Genesis::with(
            schema.schema_id(),
            chain,
            metadata.into(),
            owned_rights,
            bset![],
        );

        Contract::with(schema, None, genesis, empty!(), empty!(), empty!())
    }
}
