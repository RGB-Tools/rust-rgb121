#[macro_use]
extern crate clap;
extern crate serde_crate as serde;

use std::collections::BTreeMap;
use std::path::PathBuf;

use bitcoin::OutPoint;
use clap::Parser;
use colored::Colorize;
use lnpbp::chain::Chain;
use rgb::fungible::allocation::{AllocatedValue, OutpointValue, UtxobValue};
use rgb::{Consignment, Contract, IntoRevealedSeal, StateTransfer};
use rgb121::{Asset, Rgb121};
use stens::AsciiString;
use strict_encoding::{StrictDecode, StrictEncode};

#[derive(Parser, Clone, Debug)]
#[clap(
    name = "rgb121",
    bin_name = "rgb121",
    author,
    version,
    about = "Command-line tool for working with RGB121 fungible assets"
)]
pub struct Opts {
    /// Bitcoin network to use
    #[clap(short, long, default_value = "signet", env = "RGB_NETWORK")]
    pub network: Chain,

    /// Command to execute
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Command {
    Issue {
        /// Asset name (up to 32 characters)
        name: AsciiString,

        /// Asset description
        description: Option<AsciiString>,

        /// Precision, i.e. number of digits reserved for fractional part
        #[clap(short, long, default_value = "8")]
        precision: u8,

        /// Asset parent ID
        parent_id: Option<AsciiString>,

        /// Asset allocations, in form of <amount>@<txid>:<vout>
        allocations: Vec<OutpointValue>,
    },

    /// Prepares state transition for assets transfer.
    Transfer {
        /// File with state transfer consignment, which endpoints will act as
        /// inputs.
        consignment: PathBuf,

        /// Bitcoin transaction UTXOs which will be spent by the transfer
        #[clap(short = 'u', long = "utxo", required = true)]
        outpoints: Vec<OutPoint>,

        /// List of transfer beneficiaries
        #[clap(required = true)]
        beneficiaries: Vec<UtxobValue>,

        /// Change output; one per schema state type.
        #[clap(short, long)]
        change: Vec<AllocatedValue>,

        /// File to store state transition transferring assets to the
        /// beneficiaries and onto change outputs.
        output: PathBuf,
    },
}

fn main() -> Result<(), String> {
    let opts = Opts::parse();

    match opts.command {
        Command::Issue {
            name,
            description,
            precision,
            parent_id,
            allocations,
        } => {
            let contract = Contract::create_rgb121(
                opts.network,
                name,
                description,
                precision,
                parent_id,
                vec![],
                vec![],
                allocations,
            );

            let _asset =
                Asset::try_from(&contract).expect("create_rgb121 does not match RGB121 schema");

            eprintln!(
                "{} {}\n",
                "Contract ID:".bright_green(),
                contract.contract_id().to_string().bright_yellow()
            );

            eprintln!("{}", "Contract YAML:".bright_green());
            eprintln!("{}", serde_yaml::to_string(contract.genesis()).unwrap());

            eprintln!("{}", "Contract JSON:".bright_green());
            println!("{}\n", serde_json::to_string(contract.genesis()).unwrap());

            eprintln!("{}", "Contract source:".bright_green());
            println!("{}\n", contract);

            // eprintln!("{}", "Asset details:".bright_green());
            // eprintln!("{}\n", serde_yaml::to_string(&asset).unwrap());
        }

        Command::Transfer {
            consignment,
            outpoints,
            beneficiaries,
            change,
            output,
        } => {
            let transfer = StateTransfer::strict_file_load(consignment).unwrap();

            let asset = Asset::try_from(&transfer).unwrap();

            let beneficiaries = beneficiaries
                .into_iter()
                .map(|v| (v.seal_confidential.into(), v.value))
                .collect();
            let change = change
                .into_iter()
                .map(|v| (v.into_revealed_seal(), v.value))
                .collect();
            let outpoints = outpoints.into_iter().collect();
            let transition = asset.transfer(outpoints, beneficiaries, change).unwrap();

            transition.strict_file_save(output).unwrap();
            //consignment.strict_file_save(output).unwrap();

            println!("{}", serde_yaml::to_string(&transition).unwrap());
            println!("{}", "Success".bold().bright_green());
        }
    }

    Ok(())
}
