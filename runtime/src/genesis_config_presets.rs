use crate::{AccountId, Balance, BalancesConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys, StakingConfig, SudoConfig, UNIT};
use alloc::{vec, vec::Vec};
use frame_support::build_struct_json_patch;
use sp_consensus_babe::AuthorityId as BabeId;
use serde_json::Value;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_core::crypto::{get_public_from_string_or_panic, Ss58Codec};
use sp_core::sr25519;
use sp_genesis_builder::{self, PresetId};
use sp_runtime::Perbill;
use sp_staking::StakerStatus;

pub const ENDOWMENT: Balance = 10_000_000 * UNIT;
pub const STASH: Balance = ENDOWMENT / 10;
pub const MAX_VALIDATORS: u32 = 10;
pub const MAX_NOMINATORS: u32 = 100;
pub const MINIMUM_VALIDATOR_COUNT: u32 = 1;
pub const SLASH_REWARD_FRACTION: Perbill = Perbill::from_percent(10);

pub const CHANTO_TESTNET_PRESET: &str = "kora_chanto_testnet";

pub type Staker = (AccountId, AccountId, Balance, StakerStatus<AccountId>);

pub fn validator(account: AccountId) -> Staker {
	// validator, controller, stash, staker status
	(account.clone(), account, STASH, StakerStatus::Validator)
}

pub fn nominator(account: AccountId, targets: Vec<AccountId>) -> Staker {
	// nominator, controller, stash, staker status with targets to nominate
	(account.clone(), account, STASH, StakerStatus::Nominator(targets))
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

pub fn session_keys_from_address(sr_addr: &str, ed_addr: &str) -> SessionKeys {
	session_keys(
		GrandpaId::from_ss58check(ed_addr).expect("Bad ss58 address"),
		BabeId::from_ss58check(sr_addr).expect("Bad ss58 address"),
		ImOnlineId::from_ss58check(sr_addr).expect("Bad ss58 address"),
	)
}

pub fn authority_keys_from_seed(seed: &str) -> (AccountId, SessionKeys) {
	(
		get_public_from_string_or_panic::<sr25519::Public>(seed).into(),
		session_keys_from_seed(seed),
	)
}

pub fn authority_keys_from_address(sr_addr: &str, ed_addr: &str) -> (AccountId, SessionKeys) {
	let sr_addr_account = AccountId::from_ss58check(sr_addr).expect("Bad ss58 address");
	(
		sr_addr_account,
		session_keys_from_address(sr_addr, ed_addr),
	)
}

// Returns the genesis config presets populated with given parameters.
fn generate_genesis_config(
	initial_authorities: Vec<(AccountId, AccountId, SessionKeys)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	stakers: Vec<Staker>,
) -> Value {
	let validator_count = initial_authorities.len() as u32;

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
			max_validator_count: Some(MAX_VALIDATORS),
			max_nominator_count: Some(MAX_NOMINATORS),
			minimum_validator_count: MINIMUM_VALIDATOR_COUNT,
			invulnerables: initial_authorities
				.iter()
				.map(|x| x.0.clone())
				.collect::<Vec<_>>()
				.try_into()
				.expect("Too many invulnerable validators: upper limit is MaxInvulnerables from pallet staking config"),
			slash_reward_fraction: SLASH_REWARD_FRACTION,
			stakers,
			..Default::default()
		},
		sudo: SudoConfig { key: Some(root_key) },
	})
}

/// Return the development genesis config.
pub fn development_config_genesis() -> Value {
	let (alice, alice_session_keys) = authority_keys_from_seed("Alice");
	let (bob, _bob_session_keys) = authority_keys_from_seed("Bob");

	generate_genesis_config(
		vec![(alice.clone(), alice.clone(), alice_session_keys)],
		alice.clone(),
		vec![alice.clone(), bob.clone()],
		vec![
			validator(alice.clone()),
			nominator(bob.clone(),vec![alice.clone()]),
		],
	)
}

/// Return the local genesis config preset.
pub fn local_config_genesis() -> Value {
	let (alice, alice_session_keys) = authority_keys_from_seed("Alice");
	let (bob, bob_session_keys) = authority_keys_from_seed("Bob");
	let (charlie, _charlie_session_keys) = authority_keys_from_seed("Charlie");

	generate_genesis_config(
		vec![
			(alice.clone(), alice.clone(), alice_session_keys),
			(bob.clone(), bob.clone(), bob_session_keys)
		],
		alice.clone(),
		vec![alice.clone(), bob.clone(), charlie.clone()],
		vec![
			validator(alice.clone()),
			validator(bob.clone()),
			nominator(charlie.clone(), vec![alice.clone(), bob.clone()]),
		],
	)
}

pub fn chanto_testnet_config_genesis() -> Value {
	let default_validator_sr_addr = "5FL9Zu4bpYu9WfCed9rMXyMLpnATMkWYnJ6CT2Tij2tVrBa1";
	let default_validator_ed_addr = "5EufNDyR3KUbHBPfDDSBFiyN77DRsdECtynzhwoTkV31k5cC";
	let (default_validator, default_validator_session_keys) = authority_keys_from_address(default_validator_sr_addr, default_validator_ed_addr);

	generate_genesis_config(
		vec![(default_validator.clone(), default_validator.clone(), default_validator_session_keys)],
		default_validator.clone(),
		vec![default_validator.clone()],
		vec![validator(default_validator.clone())],
	)
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_ref() {
		sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
		sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_config_genesis(),
		CHANTO_TESTNET_PRESET => chanto_testnet_config_genesis(),
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
		PresetId::from(CHANTO_TESTNET_PRESET),
	]
}
