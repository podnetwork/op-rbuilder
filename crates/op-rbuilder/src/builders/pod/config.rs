use crate::args::OpRbuilderArgs;
use alloy_primitives::Address;

/// Configuration values that are specific to the flashblocks builder.
#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub contract_address: Address,
}

impl TryFrom<OpRbuilderArgs> for Config {
    type Error = eyre::Report;

    fn try_from(args: OpRbuilderArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            rpc_url: args.pod.pod_rpc_url,
            contract_address: args.pod.pod_contract_address,
        })
    }
}
