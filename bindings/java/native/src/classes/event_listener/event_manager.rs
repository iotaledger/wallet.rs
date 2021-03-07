use iota_wallet::{
    Error as WalletError,
    event::{
        EventId, on_error,
    },
};

pub struct EventManager {

}

pub trait ErrorEvent {
    fn on_error(&self, error: String);
}

impl EventManager {

    pub fn subscribe_errors(cb: Box<dyn ErrorEvent + Send + 'static>) -> EventId {
        on_error(move |error| {
            cb.on_error(error.to_string());
        })
    }
}