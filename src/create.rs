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
        name: AsciiString,
        description: AsciiString,
        precision: u8,
        allocations: OutpointValueVec,
        engraving: Option<OutPoint>,
    ) -> Contract;
}

impl<'consignment> Rgb21<'consignment> for Contract {
    fn create_rgb21(
        chain: Chain,
        name: AsciiString,
        description: AsciiString,
        precision: u8,
        allocations: OutpointValueVec,
        engraving: Option<OutPoint>,
    ) -> Contract {
        let now = Utc::now().timestamp();
        let mut metadata = type_map! {
            FieldType::Name => field!(AsciiString, name),
            FieldType::Description => field!(AsciiString, description),
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

        // TODO: enable engraving
        /*
        if let Some(outpoint) = engraving {
            owned_rights.insert(
                OwnedRightType::Engraving.into(),
                TypedAssignments::Attachment(vec![Assignment::Revealed {
                    seal: outpoint.into(),
                    state: rgb::contract::attachment::Revealed { },
                }]),
            );
        }
        */

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
