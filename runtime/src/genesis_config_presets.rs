// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
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

use crate::{AccountId, Balance, BalancesConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys, StakingConfig, SudoConfig, UNIT};
use alloc::{vec, vec::Vec};
use frame_support::build_struct_json_patch;
use sp_consensus_babe::AuthorityId as BabeId;
use serde_json::Value;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_core::crypto::get_public_from_string_or_panic;
use sp_core::sr25519;
use sp_genesis_builder::{self, PresetId};
use sp_runtime::Perbill;
use sp_staking::StakerStatus;
use pallet_staking::ValidatorPrefs;

pub const ENDOWMENT: Balance = 10_000_000 * UNIT;

pub type Staker = (AccountId, AccountId, Balance, StakerStatus<AccountId>);

pub fn validator(account: AccountId) -> Staker {
	// validator, controller, stash, staker status
	(account.clone(), account, ENDOWMENT, StakerStatus::Validator)
}

pub fn nominator(account: AccountId, targets: Vec<AccountId>) -> Staker {
	// nominator, controller, stash, staker status with targets to nominate
	(account.clone(), account, ENDOWMENT, StakerStatus::Nominator(targets))
}

/// Create default validator preferences with a commission rate
pub fn default_validator_prefs() -> ValidatorPrefs {
	ValidatorPrefs {
		commission: Perbill::from_percent(5),
		blocked: false,
	}
}

pub fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online }
}

pub fn session_keys_from_seed(seed: &str) -> SessionKeys {
	session_keys(
		get_public_from_string_or_panic::<GrandpaId>(seed),
		get_public_from_string_or_panic::<BabeId>(seed),
		get_public_from_string_or_panic::<ImOnlineId>(seed),
	)
}

pub fn authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, SessionKeys) {
	(
		get_public_from_string_or_panic::<sr25519::Public>(&alloc::format!("{seed}//stash")).into(),
		get_public_from_string_or_panic::<sr25519::Public>(seed).into(),
		session_keys_from_seed(seed),
	)
}

// Returns the genesis config presets populated with given parameters.
fn testnet_genesis(
	initial_authorities: Vec<(AccountId, AccountId, SessionKeys)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	stakers: Vec<Staker>,
) -> Value {
	let validator_count = initial_authorities.len() as u32;
	let minimum_validator_count = validator_count;

	build_struct_json_patch!(RuntimeGenesisConfig {
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
			..Default::default()
		},
		babe: pallet_babe::GenesisConfig {
			epoch_config: crate::apis::BABE_GENESIS_EPOCH_CONFIG,
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| { (x.0.clone(), x.1.clone(), x.2.clone()) })
				.collect(),
		},
		staking: StakingConfig {
			validator_count,
			minimum_validator_count,
			invulnerables: initial_authorities
				.iter()
				.map(|x| x.0.clone())
				.collect::<Vec<_>>()
				.try_into()
				.expect("Too many invulnerable validators: upper limit is MaxInvulnerables from pallet staking config"),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
		},
		sudo: SudoConfig { key: Some(root_key) },
	})
}

/// Return the development genesis config.
pub fn development_config_genesis() -> Value {
	let (alice_stash, alice, alice_session_keys) = authority_keys_from_seed("Alice");
	let (bob_stash, bob, _bob_session_keys) = authority_keys_from_seed("Bob");

	testnet_genesis(
		vec![(alice_stash.clone(), alice_stash.clone(), alice_session_keys)],
		alice.clone(),
		vec![alice.clone(), alice_stash.clone(), bob.clone(), bob_stash.clone()],
		vec![
			validator(alice_stash.clone()),
			nominator(bob_stash.clone(), vec![alice_stash.clone()]),
		],
	)
}

/// Return the local genesis config preset.
pub fn local_config_genesis() -> Value {
	let (alice_stash, alice, alice_session_keys) = authority_keys_from_seed("Alice");
	let (bob_stash, bob, _bob_session_keys) = authority_keys_from_seed("Bob");

	testnet_genesis(
		vec![(alice_stash.clone(), alice_stash.clone(), alice_session_keys)],
		alice.clone(),
		vec![alice.clone(), alice_stash.clone(), bob.clone(), bob_stash.clone()],
		vec![
			validator(alice_stash.clone()),
			nominator(bob_stash.clone(), vec![alice_stash.clone()]),
		],
	)
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_ref() {
		sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
		sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_config_genesis(),
		_ => return None,
	};
	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work. qed.")
			.into_bytes(),
	)
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
	vec![
		PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
		PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
	]
}
