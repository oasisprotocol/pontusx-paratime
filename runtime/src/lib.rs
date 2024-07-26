//! The Pontus-X ParaTime.
#![deny(rust_2018_idioms, single_use_lifetimes, unreachable_pub)]

use std::collections::{BTreeMap, BTreeSet};

#[cfg(target_env = "sgx")]
use oasis_runtime_sdk::core::consensus::verifier::TrustRoot;
use oasis_runtime_sdk::{
    self as sdk, config,
    core::common::crypto::signature::PublicKey,
    keymanager::TrustedSigners,
    modules,
    types::{address::Address, token::Denomination},
    Module, Version,
};
use once_cell::unsync::Lazy;

/// Configuration of the various modules.
pub struct Config;

/// The total supply of the native denomination token.
const NATIVE_TOKEN_SUPPLY: u128 = 100_000_000 * 1_000_000_000_000_000_000;

/// Determine whether the build is for Devnet.
///
/// If the crate version has a pre-release component (e.g. 3.0.0-alpha-devnet) then the build is
/// classified as Devnet if that pre-release component contains "devnet".
const fn is_devnet() -> bool {
    const_str::contains!(env!("CARGO_PKG_VERSION_PRE"), "devnet")
}

/// Determine whether the build is for Testnet.
///
/// If the crate version has a pre-release component (e.g. 3.0.0-alpha-testnet) then the build is
/// classified as Testnet if that pre-release component contains "testnet".
const fn is_testnet() -> bool {
    const_str::contains!(env!("CARGO_PKG_VERSION_PRE"), "testnet")
}

/// Determine EVM chain ID to use depending on whether the build is for Devnet, Testnet or Mainnet.
const fn chain_id() -> u64 {
    if option_env!("OASIS_UNSAFE_USE_LOCALNET_CHAINID").is_some() {
        // Localnet.
        0x7ec7
    } else if is_devnet() {
        // Devnet.
        0x7ec8
    } else if is_testnet() {
        // Testnet.
        0x7ec9
    } else {
        // Mainnet.
        panic!("runtime is not yet deployable on mainnet");
        //0x7eca
    }
}

/// Determine state version on whether the build is for Testnet or Mainnet.
const fn state_version() -> u32 {
    if is_devnet() {
        // Devnet.
        1
    } else if is_testnet() {
        // Testnet.
        1
    } else {
        // Mainnet.
        panic!("runtime is not yet deployable on mainnet");
    }
}

/// Determine the consensus denomination used by the runtime, depending on
/// whether the build is for Testner or Mainnet.
fn consensus_denomination() -> Denomination {
    if is_devnet() || is_testnet() {
        "TEST".parse().unwrap()
    } else {
        "ROSE".parse().unwrap()
    }
}

impl modules::core::Config for Config {
    /// Default local minimum gas price configuration that is used in case no overrides are set in
    /// local per-node configuration.
    const DEFAULT_LOCAL_MIN_GAS_PRICE: Lazy<BTreeMap<Denomination, u128>> = Lazy::new(|| {
        [
            (Denomination::NATIVE, 100_000_000_000),
            (consensus_denomination(), 100_000_000_000),
        ]
        .into()
    });

    /// Methods which are exempt from minimum gas price requirements.
    const MIN_GAS_PRICE_EXEMPT_METHODS: Lazy<BTreeSet<&'static str>> =
        Lazy::new(|| ["consensus.Deposit"].into());

    /// Estimated gas amount to be added to failed transaction simulations for selected methods.
    const ESTIMATE_GAS_EXTRA_FAIL: Lazy<BTreeMap<&'static str, u64>> =
        Lazy::new(|| [("evm.Create", 5_000_000), ("evm.Call", 5_000_000)].into());
}

impl module_evm::Config for Config {
    type AdditionalPrecompileSet = ();

    const CHAIN_ID: u64 = chain_id();

    const TOKEN_DENOMINATION: Denomination = Denomination::NATIVE;

    const CONFIDENTIAL: bool = true;
}

/// The EVM ParaTime.
pub struct Runtime;

impl sdk::Runtime for Runtime {
    /// Version of the runtime.
    const VERSION: Version = sdk::version_from_cargo!();
    /// Current version of the global state (e.g. parameters). Any parameter updates should bump
    /// this version in order for the migrations to be executed.
    const STATE_VERSION: u32 = state_version();

    /// Schedule control configuration.
    const SCHEDULE_CONTROL: config::ScheduleControl = config::ScheduleControl {
        initial_batch_size: 50,
        batch_size: 50,
        min_remaining_gas: 1_000, // accounts.Transfer method calls.
        max_tx_count: 1_000,      // Consistent with runtime descriptor.
    };

    type Core = modules::core::Module<Config>;
    type Accounts = modules::accounts::Module;

    #[allow(clippy::type_complexity)]
    type Modules = (
        // Core.
        modules::core::Module<Config>,
        // Accounts.
        modules::accounts::Module,
        // Consensus layer interface.
        modules::consensus::Module,
        // Consensus layer accounts.
        modules::consensus_accounts::Module<modules::consensus::Module>,
        // EVM.
        module_evm::Module<Config>,
    );

    fn trusted_signers() -> Option<TrustedSigners> {
        #[allow(clippy::partialeq_to_none)]
        if option_env!("OASIS_UNSAFE_SKIP_KM_POLICY") == Some("1") {
            return Some(TrustedSigners::default());
        }
        let tps = keymanager::trusted_policy_signers();
        // The `keymanager` crate may use a different version of `oasis_core`
        // so we need to convert the `TrustedSigners` between the versions.
        Some(TrustedSigners {
            signers: tps.signers.into_iter().map(|s| PublicKey(s.0)).collect(),
            threshold: tps.threshold,
        })
    }

    #[cfg(target_env = "sgx")]
    fn consensus_trust_root() -> Option<TrustRoot> {
        if is_devnet() {
            // Devnet.
            Some(TrustRoot {
                height: 19377991,
                hash: "99ece49085f04e312e6b55674ad700b8f9d51e1bd16ade26e2de96485ae6965a".into(),
                runtime_id: "0000000000000000000000000000000000000000000000004febe52eb412b421"
                    .into(),
                chain_context: "0b91b8e4e44b2003a7c5e23ddadb5e14ef5345c0ebcb3ddcae07fa2f244cab76"
                    .to_string(),
            })
        } else if is_testnet() {
            // Testnet.
            Some(TrustRoot {
                height: 21263648,
                hash: "1b2ac553253818973cd3d1b8b5fc856e3be3a68848cb5b8aaf280f7a4fd92c7c".into(),
                runtime_id: "00000000000000000000000000000000000000000000000004a6f9071c007069"
                    .into(),
                chain_context: "0b91b8e4e44b2003a7c5e23ddadb5e14ef5345c0ebcb3ddcae07fa2f244cab76"
                    .to_string(),
            })
        } else {
            panic!("runtime is not yet deployable on mainnet");
            // Mainnet.
            /*Some(TrustRoot {
                height: 0, // TODO
                hash: "".into(), // TODO
                runtime_id: "000000000000000000000000000000000000000000000000b2406b4af4e630fe"
                    .into(),
                chain_context: "bb3d748def55bdfb797a2ac53ee6ee141e54cd2ab2dc2375f4a0703a178e6e55"
                    .to_string(),
            })*/
        }
    }

    fn genesis_state() -> <Self::Modules as sdk::module::MigrationHandler>::Genesis {
        (
            modules::core::Genesis {
                parameters: modules::core::Parameters {
                    min_gas_price: {
                        BTreeMap::from([
                            (Denomination::NATIVE, 100_000_000_000),
                            (consensus_denomination(), 100_000_000_000),
                        ])
                    },
                    dynamic_min_gas_price: modules::core::DynamicMinGasPrice {
                        enabled: true,
                        target_block_gas_usage_percentage: 50,
                        min_price_max_change_denominator: 8,
                    },
                    max_batch_gas: 15_000_000,
                    max_tx_size: 128 * 1024,
                    max_tx_signers: 1,
                    max_multisig_signers: 8,
                    gas_costs: modules::core::GasCosts {
                        tx_byte: 1,
                        storage_byte: 0,
                        auth_signature: 1_000,
                        auth_multisig_signer: 1_000,
                        callformat_x25519_deoxysii: 10_000,
                    },
                },
            },
            modules::accounts::Genesis {
                parameters: modules::accounts::Parameters {
                    gas_costs: modules::accounts::GasCosts { tx_transfer: 1_000 },
                    denomination_infos: {
                        BTreeMap::from([
                            (
                                Denomination::NATIVE,
                                modules::accounts::types::DenominationInfo {
                                    // Consistent with EVM ecosystem.
                                    decimals: 18,
                                },
                            ),
                            (
                                consensus_denomination(),
                                modules::accounts::types::DenominationInfo { decimals: 18 },
                            ),
                        ])
                    },
                    ..Default::default()
                },
                balances: BTreeMap::from([(
                    Address::from_eth(
                        &hex::decode("24D68bFBA0fB06ccFfD21dC3a5c0B65207Bd479a").unwrap(),
                    ),
                    BTreeMap::from([(Denomination::NATIVE, NATIVE_TOKEN_SUPPLY)]),
                )]),
                total_supplies: BTreeMap::from([(Denomination::NATIVE, NATIVE_TOKEN_SUPPLY)]),
                ..Default::default()
            },
            modules::consensus::Genesis {
                parameters: modules::consensus::Parameters {
                    gas_costs: modules::consensus::GasCosts { round_root: 10_000 },
                    // Consensus layer denomination is the native denomination of this runtime.
                    consensus_denomination: consensus_denomination(),
                    // Scale to 18 decimal places as this is what is expected in the EVM ecosystem.
                    consensus_scaling_factor: 1_000_000_000,
                    // Minimum delegation amount that matches the consensus layer.
                    min_delegate_amount: 100_000_000_000,
                },
            },
            modules::consensus_accounts::Genesis {
                parameters: modules::consensus_accounts::Parameters {
                    gas_costs: modules::consensus_accounts::GasCosts {
                        tx_deposit: 60_000,
                        tx_withdraw: 60_000,
                        tx_delegate: 60_000,
                        tx_undelegate: 120_000,

                        store_receipt: 20_000,
                        take_receipt: 15_000,
                    },
                    disable_delegate: false,
                    disable_undelegate: false,
                    disable_deposit: false,
                    disable_withdraw: false,
                },
            },
            module_evm::Genesis {
                parameters: module_evm::Parameters {
                    gas_costs: module_evm::GasCosts {},
                },
            },
        )
    }

    fn migrate_state<C: sdk::Context>(_ctx: &C) {
        // State migration from by copying over parameters from updated genesis state.
        let genesis = Self::genesis_state();

        // Core.
        modules::core::Module::<Config>::set_params(genesis.0.parameters);
        // Accounts.
        modules::accounts::Module::set_params(genesis.1.parameters);
        // Consensus layer interface.
        modules::consensus::Module::set_params(genesis.2.parameters);
        // Consensus layer accounts.
        modules::consensus_accounts::Module::<modules::consensus::Module>::set_params(
            genesis.3.parameters,
        );
        // EVM.
        module_evm::Module::<Config>::set_params(genesis.4.parameters);
    }
}
