use crate::context::RpcContext;
use anyhow::Context;
use pathfinder_common::TransactionHash;

use starknet_gateway_types::reply::transaction::Transaction as GatewayTransaction;

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetTransactionByHashInput {
    transaction_hash: TransactionHash,
}

crate::error::generate_rpc_error_subset!(GetTransactionByHashError: TxnHashNotFoundV03);

pub async fn get_transaction_by_hash_impl(
    context: RpcContext,
    input: GetTransactionByHashInput,
) -> anyhow::Result<Option<GatewayTransaction>> {
    let storage = context.storage.clone();
    let span = tracing::Span::current();

    let jh = tokio::task::spawn_blocking(move || {
        let _g = span.enter();
        let mut db = storage
            .connection()
            .context("Opening database connection")?;

        let db_tx = db.transaction().context("Creating database transaction")?;

        // Check pending transactions.
        if let Some(tx) = context
            .pending_data
            .get(&db_tx)
            .context("Querying pending data")?
            .block
            .transactions
            .iter()
            .find(|tx| tx.hash() == input.transaction_hash)
            .cloned()
        {
            return Ok(Some(tx));
        }

        // Get the transaction from storage.
        db_tx
            .transaction(input.transaction_hash)
            .context("Reading transaction from database")
    });

    jh.await.context("Database read panic or shutting down")?
}

#[cfg(test)]
mod tests {
    use super::*;
    use pathfinder_common::macro_prelude::*;

    mod parsing {
        use super::*;
        use serde_json::json;

        #[test]
        fn positional_args() {
            let positional = json!(["0xdeadbeef"]);

            let input = serde_json::from_value::<GetTransactionByHashInput>(positional).unwrap();
            assert_eq!(
                input,
                GetTransactionByHashInput {
                    transaction_hash: transaction_hash!("0xdeadbeef")
                }
            )
        }

        #[test]
        fn named_args() {
            let named_args = json!({
                "transaction_hash": "0xdeadbeef"
            });
            let input = serde_json::from_value::<GetTransactionByHashInput>(named_args).unwrap();
            assert_eq!(
                input,
                GetTransactionByHashInput {
                    transaction_hash: transaction_hash!("0xdeadbeef")
                }
            )
        }
    }
}
