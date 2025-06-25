use config::Config;
use payload::PodPayloadBuilderBuilder;
use reth_node_builder::components::BasicPayloadServiceBuilder;

use crate::traits::{NodeBounds, PoolBounds};

use super::BuilderConfig;

mod config;
mod payload;
mod pod_client;

/// Block building strategy that builds blocks using auction running on pod by
/// producing blocks every chain block time.
pub struct PodBuilder;

impl super::PayloadBuilder for PodBuilder {
    type Config = Config;

    type ServiceBuilder<Node, Pool>
        = BasicPayloadServiceBuilder<PodPayloadBuilderBuilder>
    where
        Node: NodeBounds,
        Pool: PoolBounds;

    fn new_service<Node, Pool>(
        config: BuilderConfig<Self::Config>,
    ) -> eyre::Result<Self::ServiceBuilder<Node, Pool>>
    where
        Node: NodeBounds,
        Pool: PoolBounds,
    {
        Ok(BasicPayloadServiceBuilder::new(PodPayloadBuilderBuilder(
            config,
        )))
    }
}
