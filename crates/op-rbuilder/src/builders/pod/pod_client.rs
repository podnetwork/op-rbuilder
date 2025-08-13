use std::{collections::VecDeque, fmt::Debug, time::Duration};

use eyre::Context;
use alloy_consensus::Transaction;
use alloy_primitives::{Address, U256};
use alloy_rlp::Decodable;
use itertools::Itertools;
use pod_sdk::{auctions::client::AuctionClient, provider::PodProviderBuilder};
use reth_optimism_primitives::OpTransactionSigned;
use reth_optimism_txpool::OpPooledTransaction;
use reth_payload_util::PayloadTransactions;
use reth_primitives::Recovered;
use tokio::sync::OnceCell;

use crate::tx::FBPooledTransaction;

pub(super) struct PodClient {
    client: OnceCell<AuctionClient>,
    rpc_url: String,
    contract: Address,
}

#[derive(Debug)]
pub(super) struct Transactions(VecDeque<FBPooledTransaction>);

impl PodClient {
    pub async fn new(rpc_url: String, contract: Address) -> eyre::Result<Self> {
        Ok(Self {
            client: OnceCell::new(),
            rpc_url,
            contract,
        })
    }

    async fn get_client(&self) -> eyre::Result<&AuctionClient> {
        self.client
            .get_or_try_init(|| async {
                let provider = PodProviderBuilder::new()
                    .on_url(self.rpc_url.clone())
                    .await
                    .map_err(eyre::Error::from)?;

                Ok(AuctionClient::new(provider, self.contract))
            })
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn best_transactions(&self, timestamp_secs: u64) -> eyre::Result<Transactions> {
        let auction = self.get_client().await?;
        let auction_deadline = timestamp_secs - 2;
        tracing::trace!(target: "payload_builder", auction_deadline, "querying best transactions");
        let auction_deadline = pod_sdk::Timestamp::from_seconds(auction_deadline);

        tracing::info!("waiting for past perfect time");
        // FIXME: The current PPT implementation in pod is dummy and waits 5s which is far too long.
        // Use auction.wait_for_auction_end() when it's fixed.
        auction
            .auction
            .provider()
            .wait_past_perfect_time(
                auction_deadline - Duration::from_secs(5) + Duration::from_millis(200),
            )
            .await
            .context("waiting for auction end (past perfect time)")?;
        tracing::info!("past perfect time reached");

        let bids = auction
            .fetch_bids_for_deadline(auction_deadline.into())
            // .fetch_bids(U256::from(auction_deadline )) // auction ID is in microseconds
            .await
            .unwrap_or_else(|err| {
                tracing::error!(target: "payload_builder", ?err, "failed to fetch bids from pod");
                Vec::new()
            });
        let transactions = bids
            .into_iter()
            .sorted_unstable_by_key(|bid| {
                bid.amount
            })
            .rev()
            .filter_map(|bid| {
                let recovered = match
                    Recovered::<OpTransactionSigned>::decode(&mut bid.data.as_slice()) {
                        Ok(tx) => {
                            tracing::info!(target: "payload_builder", tx=%tx.tx_hash(), bid=%bid.amount, "fetched tx from pod: {tx:?}");
                            Some(tx)
                        },
                        Err(error) => {
                            tracing::warn!(target: "payload_builder", ?error, "failed to decode transaction from pod");
                            None
                        }
                    }?;
                if U256::from(recovered.max_priority_fee_per_gas().unwrap_or(0)) != bid.amount {
                            tracing::error!(target: "payload_builder", tx=%recovered.tx_hash(), "ignoring tx with different max priority fee per gas than bid amount");
                }

                Some(OpPooledTransaction::new(recovered, bid.data.len()).into())
            })
            .collect::<VecDeque<_>>();
        Ok(Transactions(transactions))
    }
}

impl Debug for PodClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodClient")
            .field("rpc_url", &self.rpc_url)
            .finish()
    }
}

impl PayloadTransactions for Transactions {
    type Transaction = FBPooledTransaction;
    /// Returns the next transaction to include in the block.
    fn next(
        &mut self,
        // In the future, `ctx` can include access to state for block building purposes.
        _ctx: (),
    ) -> Option<Self::Transaction> {
        self.0.pop_front()
    }

    /// Exclude descendants of the transaction with given sender and nonce from the iterator,
    /// because this transaction won't be included in the block.
    fn mark_invalid(&mut self, sender: Address, nonce: u64) {
        tracing::warn!(
            target: "payload_builder",
            %sender,
            nonce,
            "mark_invalid called"
        );
    }
}

impl Transactions {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
