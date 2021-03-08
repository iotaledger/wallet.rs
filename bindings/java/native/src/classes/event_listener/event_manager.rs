use iota_wallet::{
    Error as WalletError,
    event::{
        EventId, 
        on_error, on_new_transaction, 
        on_confirmation_state_change, on_reattachment, on_broadcast, on_transfer_progress, on_balance_change,
        TransactionEvent as WalletTransactionEvent,
        TransactionConfirmationChangeEvent as WalletTransactionConfirmationChangeEvent,
        TransferProgress as WalletTransferProgress,
        BalanceEvent as WalletBalanceEvent,
    },
};

pub struct EventManager {

}

pub trait ErrorListener {
    fn on_error(&self, error: String);
}

pub trait NewTransactionListener {
    fn on_new_transaction(&self, error: WalletTransactionEvent);
}

pub trait ReattachTransactionListener {
    fn on_reattachment(&self, error: WalletTransactionEvent);
}

pub trait BroadcastTransactionListener {
    fn on_broadcast(&self, error: WalletTransactionEvent);
}

pub trait TransactionConfirmationChangeListener {
    fn on_confirmation_state_change(&self, error: WalletTransactionConfirmationChangeEvent);
}

pub trait TransferProgressListener {
    fn on_transfer_progress(&self, error: WalletTransferProgress);
}

pub trait BalanceChangeListener {
    fn on_balance_change(&self, error: WalletBalanceEvent);
}

impl EventManager {

    pub fn subscribe_new_transaction(cb: Box<dyn NewTransactionListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            on_new_transaction(move |event| {
                cb.on_new_transaction(event.clone());
            }).await
        })
    }

    pub fn subscribe_confirmation_state_change(cb: Box<dyn TransactionConfirmationChangeListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            on_confirmation_state_change(move |event| {
                cb.on_confirmation_state_change(event.clone());
            }).await
        })
    }
    
    pub fn subscribe_reattachment(cb: Box<dyn ReattachTransactionListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            on_reattachment(move |event| {
                cb.on_reattachment(event.clone());
            }).await
        })
    }
    
    pub fn subscribe_broadcast(cb: Box<dyn BroadcastTransactionListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            on_broadcast(move |event| {
                cb.on_broadcast(event.clone());
            }).await
        })
    }

    pub fn subscribe_transfer_progress(cb: Box<dyn TransferProgressListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            on_transfer_progress(move |event| {
                cb.on_transfer_progress(event.clone());
            }).await
        })
    }

    pub fn subscribe_balance_change(cb: Box<dyn BalanceChangeListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            on_balance_change(move |event| {
                cb.on_balance_change(event.clone());
            }).await
        })
    }

    pub fn subscribe_errors(cb: Box<dyn ErrorListener + Send + 'static>) -> EventId {
        on_error(move |error| {
            cb.on_error(error.to_string());
        })
    }
}