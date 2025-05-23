use sc_service::ChainType;
use kora_chain_runtime::WASM_BINARY;
use serde::{Deserialize, Serialize};
use sc_chain_spec::ChainSpecExtension;
use sc_sync_state_rpc::LightSyncStateExtension;

#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	light_sync_state: LightSyncStateExtension,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

pub fn development_chain_spec() -> Result<ChainSpec, String> {
	let mut properties = sc_service::Properties::new();
	properties.insert("tokenSymbol".into(), "KORA".into());
	properties.insert("tokenDecimals".into(), 12.into());

	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		Extensions::default(),
	)
	.with_name("Development")
	.with_id("dev")
	.with_properties(properties)
	.with_chain_type(ChainType::Development)
	.with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
	.build())
}

pub fn local_chain_spec() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		Extensions::default(),
	)
	.with_name("Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)
	.build())
}
