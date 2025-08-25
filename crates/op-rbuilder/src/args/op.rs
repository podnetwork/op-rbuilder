//! Additional Node command arguments.
//!
//! Copied from OptimismNode to allow easy extension.

//! clap [Args](clap::Args) for optimism rollup configuration

use crate::{flashtestations::args::FlashtestationsArgs, tx_signer::Signer};
use alloy_primitives::Address;
use anyhow::{anyhow, Result};
use clap::Parser;
use reth_optimism_cli::commands::Commands;
use reth_optimism_node::args::RollupArgs;
use std::path::PathBuf;

/// Parameters for rollup configuration
#[derive(Debug, Clone, PartialEq, Eq, clap::Args)]
#[command(next_help_heading = "Rollup")]
pub struct OpRbuilderArgs {
    /// Rollup configuration
    #[command(flatten)]
    pub rollup_args: RollupArgs,
    /// Builder secret key for signing last transaction in block
    #[arg(long = "rollup.builder-secret-key", env = "BUILDER_SECRET_KEY")]
    pub builder_signer: Option<Signer>,

    /// chain block time in milliseconds
    #[arg(
        long = "rollup.chain-block-time",
        default_value = "1000",
        env = "CHAIN_BLOCK_TIME"
    )]
    pub chain_block_time: u64,

    /// Signals whether to log pool transaction events
    #[arg(long = "builder.log-pool-transactions", default_value = "false")]
    pub log_pool_transactions: bool,

    /// How much time extra to wait for the block building job to complete and not get garbage collected
    #[arg(long = "builder.extra-block-deadline-secs", default_value = "20")]
    pub extra_block_deadline_secs: u64,
    /// Whether to enable revert protection by default
    #[arg(long = "builder.enable-revert-protection", default_value = "false")]
    pub enable_revert_protection: bool,

    /// Path to builder playgorund to automatically start up the node connected to it
    #[arg(
        long = "builder.playground",
        num_args = 0..=1,
        default_missing_value = "$HOME/.playground/devnet/",
        value_parser = expand_path,
        env = "PLAYGROUND_DIR",
    )]
    pub playground: Option<PathBuf>,
    #[command(flatten)]
    pub pod: PodArgs,
    #[command(flatten)]
    pub flashblocks: FlashblocksArgs,
    #[command(flatten)]
    pub telemetry: TelemetryArgs,
    #[command(flatten)]
    pub flashtestations: FlashtestationsArgs,
}

impl Default for OpRbuilderArgs {
    fn default() -> Self {
        let args = crate::args::Cli::parse_from(["dummy", "node"]);
        let Commands::Node(node_command) = args.command else {
            unreachable!()
        };
        node_command.ext
    }
}

fn expand_path(s: &str) -> Result<PathBuf> {
    shellexpand::full(s)
        .map_err(|e| anyhow!("expansion error for `{s}`: {e}"))?
        .into_owned()
        .parse()
        .map_err(|e| anyhow!("invalid path after expansion: {e}"))
}

/// Parameters for Flashblocks configuration
/// The names in the struct are prefixed with `flashblocks` to avoid conflicts
/// with the standard block building configuration since these args are flattened
/// into the main `OpRbuilderArgs` struct with the other rollup/node args.
#[derive(Debug, Clone, PartialEq, Eq, clap::Args)]
pub struct FlashblocksArgs {
    /// When set to true, the builder will build flashblocks
    /// and will build standard blocks at the chain block time.
    ///
    /// The default value will change in the future once the flashblocks
    /// feature is stable.
    #[arg(
        long = "flashblocks.enabled",
        default_value = "false",
        env = "ENABLE_FLASHBLOCKS"
    )]
    pub enabled: bool,

    /// The port that we bind to for the websocket server that provides flashblocks
    #[arg(
        long = "flashblocks.port",
        env = "FLASHBLOCKS_WS_PORT",
        default_value = "1111"
    )]
    pub flashblocks_port: u16,

    /// The address that we bind to for the websocket server that provides flashblocks
    #[arg(
        long = "flashblocks.addr",
        env = "FLASHBLOCKS_WS_ADDR",
        default_value = "127.0.0.1"
    )]
    pub flashblocks_addr: String,

    /// flashblock block time in milliseconds
    #[arg(
        long = "flashblocks.block-time",
        default_value = "250",
        env = "FLASHBLOCK_BLOCK_TIME"
    )]
    pub flashblocks_block_time: u64,

    /// Builder would always thry to produce fixed number of flashblocks without regard to time of
    /// FCU arrival.
    /// In cases of late FCU it could lead to partially filled blocks.
    #[arg(
        long = "flashblocks.fixed",
        default_value = "false",
        env = "FLASHBLOCK_FIXED"
    )]
    pub flashblocks_fixed: bool,

    /// Time by which blocks would be completed earlier in milliseconds.
    ///
    /// This time used to account for latencies, this time would be deducted from total block
    /// building time before calculating number of fbs.
    #[arg(
        long = "flashblocks.leeway-time",
        default_value = "75",
        env = "FLASHBLOCK_LEEWAY_TIME"
    )]
    pub flashblocks_leeway_time: u64,
}

impl Default for FlashblocksArgs {
    fn default() -> Self {
        let args = crate::args::Cli::parse_from(["dummy", "node"]);
        let Commands::Node(node_command) = args.command else {
            unreachable!()
        };
        node_command.ext.flashblocks
    }
}

#[derive(Debug, Clone, PartialEq, Eq, clap::Args)]
pub struct PodArgs {
    #[arg(long = "pod.enabled", default_value = "false", env = "ENABLE_POD")]
    pub is_enabled: bool,

    #[arg(
        long = "pod.rpc-url",
        env = "POD_RPC_URL",
        default_value = "wss://rpc.v2.pod.network"
    )]
    pub pod_rpc_url: String,

    #[arg(
        long = "pod.contract-address",
        env = "POD_CONTRACT_ADDRESS",
        default_value = "0xedd0670497e00ded712a398563ea938a29dd28c7"
    )]
    pub pod_contract_address: Address,
}

/// Parameters for telemetry configuration
#[derive(Debug, Clone, Default, PartialEq, Eq, clap::Args)]
pub struct TelemetryArgs {
    /// OpenTelemetry endpoint for traces
    #[arg(long = "telemetry.otlp-endpoint", env = "OTEL_EXPORTER_OTLP_ENDPOINT")]
    pub otlp_endpoint: Option<String>,

    /// OpenTelemetry headers for authentication
    #[arg(long = "telemetry.otlp-headers", env = "OTEL_EXPORTER_OTLP_HEADERS")]
    pub otlp_headers: Option<String>,

    /// Inverted sampling frequency in blocks. 1 - each block, 100 - every 100th block.
    #[arg(
        long = "telemetry.sampling-ratio",
        env = "SAMPLING_RATIO",
        default_value = "100"
    )]
    pub sampling_ratio: u64,
}
