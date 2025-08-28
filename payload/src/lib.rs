use primitives::handle::{PayloadBuilderHandleMessage, PayloadBuilderResultMessage};
use provider::{Database, ProviderFactory};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use transaction_pool::Pool;

use crate::handle::{PayloadBuilderHandle};

pub mod handle;

#[derive(Clone, Debug)]
pub struct PayloadBuilder<DB: Database> {
    provider: ProviderFactory<DB>,
    pool: Pool<DB>,
}

impl<DB: Database> PayloadBuilder<DB> {
    pub fn new(provider: ProviderFactory<DB>, pool: Pool<DB>) -> Self {
        Self {
            provider,
            pool
        }
    }

    pub fn start_orchestration(self) -> (PayloadBuilderHandle, UnboundedReceiver<PayloadBuilderResultMessage>) {
        let (to_manager_tx, to_manager_rx) = mpsc::unbounded_channel::<PayloadBuilderHandleMessage>();
        let (orchestration_tx, builder_rx) = mpsc::unbounded_channel::<PayloadBuilderResultMessage>();

        let builder_handle = PayloadBuilderHandle::new(to_manager_tx);

        self.start_channel(to_manager_rx, orchestration_tx);

        (builder_handle, builder_rx)
    }

    fn start_channel(
        self, 
        mut to_manager_rx: UnboundedReceiver<PayloadBuilderHandleMessage>, 
        mut orchestration_tx: UnboundedSender<PayloadBuilderResultMessage>
    ) {
        tokio::spawn(async move {
            println!("PayloadBuilder channel starts.");
            let PayloadBuilder { provider, pool } = self;

            loop {
                if let Some(msg) = to_manager_rx.recv().await {
                    println!("PayloadBuilder received message: {:?}", msg);
                    match msg {
                        PayloadBuilderHandleMessage::BuildPayload => {

                        }
                        PayloadBuilderHandleMessage::Stop => {

                        }
                    }
                }
            }
        });
    }
}