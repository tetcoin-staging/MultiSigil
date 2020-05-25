// Copyright © 2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::{clap_app, crate_authors, crate_description};
use codec::{Decode as _, Encode as _};
use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
use sp_io::hashing::blake2_256;
use sp_runtime::AccountId32 as AccountId;

struct NetworkPasringError;

impl std::fmt::Display for NetworkPasringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NetworkParsingError")
    }
}

#[derive(Debug)]
enum Network {
    Polkadot,
    Kusama,
}

impl std::str::FromStr for Network {
    type Err = NetworkPasringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "polkadot" => Ok(Network::Polkadot),
            "kusama" => Ok(Network::Kusama),
            _ => Err(NetworkPasringError),
        }
    }
}

impl Network {
    fn format(&self) -> Ss58AddressFormat {
        match self {
            Network::Polkadot => Ss58AddressFormat::PolkadotAccount,
            Network::Kusama => Ss58AddressFormat::KusamaAccount,
        }
    }
}

struct AddressParsingError;

impl std::fmt::Display for AddressParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AddressPasringError")
    }
}

struct Address {
    account: AccountId,
    format: Ss58AddressFormat,
}

impl std::str::FromStr for Address {
    type Err = AddressParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <AccountId as Ss58Codec>::from_string_with_version(s)
            .map_err(|_| AddressParsingError{})
            .map(|(account, format)| Self { account, format })
    }
}

fn main() {
    let matches = clap_app!((env!("CARGO_PKG_NAME")) =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: crate_authors!("\n"))
        (about: crate_description!())
        (@arg NETWORK: --network +takes_value possible_value[kusama polkadot] default_value[kusama] "Network to calculate multisig for; defaults to Kusama")
        (@arg THRESHOLD: +required "The number of signatures needed to perform the operation")
        (@arg ADDRESSES: +required ... "The addresses to use")
    )
    .get_matches();

    let network: Network = matches
        .value_of_t("NETWORK")
        .expect("NETWORK has a default value");
    let threshold: u16 = matches
        .value_of_t("THRESHOLD")
        .expect("THRESHOLD is a required argument");
    let mut who: Vec<AccountId> = matches
        .values_of_t::<Address>("ADDRESSES")
        .expect("ADDRESSES are required arguments")
        .into_iter()
        .map(|address| {
            if address.format == network.format() {
                address.account
            } else {
                panic!(
                    "Address type mismatch, please make sure to only specify {:?} addresses",
                    network
                )
            }
        })
        .collect();
    who.sort_unstable();
    let entropy = (b"modlpy/utilisuba", who, threshold).using_encoded(blake2_256);
    println!(
        "{}",
        AccountId::decode(&mut &entropy[..])
            .unwrap_or_default()
            .to_ss58check_with_version(network.format())
    )
}
