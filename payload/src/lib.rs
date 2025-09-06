use std::time::{SystemTime, UNIX_EPOCH};

use primitives::{block::{Payload, PayloadHeader}, handle::{PayloadBuilderHandleMessage, PayloadBuilderResultMessage}, merkle::calculate_merkle_root,types::{Address, U256}};
use provider::{executor::Executor, Database, ProviderFactory};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use transaction_pool::Pool;

use crate::{builder::{BuildArguments}, error::PayloadBuilderError, handle::PayloadBuilderHandle};

pub mod handle; 
pub mod error;
pub mod builder;

#[derive(Debug)]
pub struct PayloadBuilder<DB: Database> {
    address: Address,
    provider: ProviderFactory<DB>,
    pool: Pool<DB>,
}

impl<DB: Database> PayloadBuilder<DB> {
    pub fn new(address: Address, provider: ProviderFactory<DB>, pool: Pool<DB>) -> Self {
        Self {
            address,
            provider,
            pool,
        }
    }

    pub fn start_builder(self) -> (PayloadBuilderHandle, UnboundedReceiver<PayloadBuilderResultMessage>) {
        let (to_manager_tx, to_manager_rx) = mpsc::unbounded_channel::<PayloadBuilderHandleMessage>();
        let (orchestration_tx, builder_rx) = mpsc::unbounded_channel::<PayloadBuilderResultMessage>();

        let builder_handle = PayloadBuilderHandle::new(to_manager_tx);

        self.start_channel(to_manager_rx, orchestration_tx);

        (builder_handle, builder_rx)
    }

    fn start_channel(
        self, 
        mut to_manager_rx: UnboundedReceiver<PayloadBuilderHandleMessage>, 
        orchestration_tx: UnboundedSender<PayloadBuilderResultMessage>
    ) {
        tokio::spawn(async move {
            println!("PayloadBuilder channel starts.");
            let PayloadBuilder { address, provider, pool } = self;

            loop {
                if let Some(msg) = to_manager_rx.recv().await {
                    println!("PayloadBuilder received message: {:?}", msg);
                    match msg {
                        PayloadBuilderHandleMessage::BuildPayload => {
                            println!("(BuildPayload) Accepted message");
                            let provider = provider.clone();
                            let pool = pool.clone();
                            let orchestration_tx = orchestration_tx.clone();

                            let parent_header = provider.db().get_latest_block_header();
                            tokio::spawn(async move {
                                match default_paylod(BuildArguments::new(address, parent_header), provider, pool).await {
                                    Ok(payload) => {
                                        if let Err(e) = orchestration_tx.send(PayloadBuilderResultMessage::Payload(payload)) {
                                            eprintln!("(BuildPayload) Failed to send PayloadBuilderResultMessage: {:?}", e);
                                        };
                                    }
                                    Err(e) => {
                                        eprintln!("(BuildPayload) Failed to make new payload {:?}", e);
                                    }
                                }
                                
                            });
                        }
                        PayloadBuilderHandleMessage::Stop => {

                        }
                    }
                }
            }
        });
    }
}

async fn default_paylod<DB: Database>(
    args: BuildArguments,
    provider: ProviderFactory<DB>, 
    pool: Pool<DB>
) -> Result<Payload, PayloadBuilderError> {
    let BuildArguments {
        address, 
        parent_header,
        attributes
    } = args;

    let state_provider = provider.latest();
    let exec_state = state_provider.executable_state()?;
    let max_transactions = attributes.max_transactions;

    let mut executor = Executor {
        state: exec_state,
        receipts: Vec::new(),
    };

    let mut best_txs = pool.best_transactions();
    let mut body = Vec::new();
    let mut total_fee = U256::ZERO;

    let mut count: u32 = 0;    

    while let Some(pool_tx) = best_txs.next() {
        match executor.execute_transaction(&pool_tx.transaction) {
            Ok(receipt) => {
                if receipt.success {
                    total_fee += U256::from(receipt.fee);
                    body.push(pool_tx.tx().tx().clone());
                }
                if count >= max_transactions {
                    break;
                }
                count += 1;
            }
            Err(_e) => {}
        }
    }

    let next_height = parent_header.height + 1;
    let tx_hashes = body.iter().map(|tx| tx.hash).collect();
    let transaction_root = calculate_merkle_root(tx_hashes);
    let state_root = executor.calculate_state_root();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time shuld go forward").as_secs();

    let payload_header = PayloadHeader {
        previous_hash: parent_header.calculate_hash(),
        transaction_root,
        state_root,
        proposer: address,
        difficulty: attributes.next_difficulty,
        height: next_height,
        timestamp,
        total_fee
    };

    let payload = Payload {
        header: payload_header,
        body: body,
    };

    Ok(payload)
}