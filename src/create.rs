use std::collections::BTreeMap;
use std::path::PathBuf;

use bitcoin::hashes::{sha256, Hash};
use bitcoin::OutPoint;
use chrono::Utc;
use commit_verify::commit_verify::CommitVerify;
use lnpbp::chain::Chain;
use rgb::fungible::allocation::{AllocationMap, IntoSealValueMap, OutpointValueVec};
use rgb::{
    secp256k1zkp, value, Assignment, AttachmentId, Consignment, Contract, Genesis, TypedAssignments,
};
use stens::AsciiString;

use crate::schema;
use crate::schema::{FieldType, OwnedRightType};

/// File to attach to genesis transition
pub struct FileAttachment {
    /// Path to the media file
    pub file_path: PathBuf,
    /// Mime of the file
    pub mime: AsciiString,
    /// NFT Salt
    pub salt: u64,
}

/// Extension trait for consignments defining RGB121-specific API.
#[allow(clippy::too_many_arguments)]
pub trait Rgb121<'consignment>: Consignment<'consignment> {
    /// Performs primary asset issue, producing [`Contract`] consignment.
    fn create_rgb121(
        chain: Chain,
        name: AsciiString,
        description: Option<AsciiString>,
        precision: u8,
        parent_id: Option<AsciiString>,
        file_attachments: Vec<FileAttachment>,
        bytes_data_vec: Vec<Vec<u8>>,
        allocations: OutpointValueVec,
    ) -> Result<Contract, Error>;
}

impl<'consignment> Rgb121<'consignment> for Contract {
    fn create_rgb121(
        chain: Chain,
        name: AsciiString,
        description: Option<AsciiString>,
        precision: u8,
        parent_id: Option<AsciiString>,
        file_attachments: Vec<FileAttachment>,
        bytes_data_vec: Vec<Vec<u8>>,
        allocations: OutpointValueVec,
    ) -> Result<Contract, Error> {
        let now = Utc::now().timestamp();
        let mut metadata = type_map! {
            FieldType::Name => field!(AsciiString, name),
            FieldType::Precision => field!(U8, precision),
            FieldType::Timestamp => field!(I64, now)
        };

        if let Some(desc) = description {
            metadata.insert(FieldType::Description.into(), field!(AsciiString, desc));
        };

        if let Some(pid) = parent_id {
            metadata.insert(FieldType::ParentId.into(), field!(AsciiString, pid));
        };

        let data: Vec<_> = bytes_data_vec
            .into_iter()
            .map(::rgb::data::Revealed::Bytes)
            .collect();
        metadata.insert(FieldType::Data.into(), data);

        let issued_supply = allocations.sum();
        let mut owned_rights = BTreeMap::new();
        owned_rights.insert(
            OwnedRightType::Assets.into(),
            TypedAssignments::zero_balanced(
                vec![value::Revealed {
                    value: issued_supply,
                    blinding: secp256k1zkp::key::ONE_KEY.into(),
                }],
                allocations.clone().into_seal_value_map(),
                empty![],
            ),
        );
        metadata.insert(FieldType::IssuedSupply.into(), field!(U64, issued_supply));

        let outpoints: Vec<OutPoint> = allocations.iter().map(|a| a.outpoint).collect();
        for file in file_attachments {
            let file_bytes = std::fs::read(file.file_path.clone()).map_err(|_| {
                Error::InvalidFileAttachment(file.file_path.to_string_lossy().to_string())
            })?;
            let file_hash = sha256::Hash::hash(&file_bytes[..]);
            let attachment_id = AttachmentId::commit(&file_hash);

            for outpoint in outpoints.clone() {
                owned_rights.insert(
                    OwnedRightType::Engraving.into(),
                    TypedAssignments::Attachment(vec![Assignment::Revealed {
                        seal: outpoint.into(),
                        state: rgb::contract::attachment::Revealed {
                            id: attachment_id,
                            mime: file.mime.clone(),
                            salt: file.salt,
                        },
                    }]),
                );
            }
        }

        let schema = schema::schema();

        let genesis = Genesis::with(
            schema.schema_id(),
            chain,
            metadata.into(),
            owned_rights,
            bset![],
        );

        Ok(Contract::with(
            schema,
            None,
            genesis,
            empty!(),
            empty!(),
            empty!(),
        ))
    }
}

/// Errors generated during RGB121 asset creation
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// The provided file attachment {0} is invalid
    InvalidFileAttachment(String),
}
