// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{block_on, event_manager::EventType};

use super::{
    destroy as destroy_actor, init as init_actor, init_logger, listen as add_event_listener,
    send_message as send_actor_message,
};

use bee_common::logger::{LoggerConfig, LoggerOutputConfigBuilder};
use log::LevelFilter;

pub struct Actor {}

pub trait ActorCallback {
    fn on_event(&self, event: &str);
}

impl Actor {
    pub fn iota_initialize(
        callback: Box<dyn ActorCallback + Send + 'static>,
        actor_id: &str,
        storage_path: Option<&str>,
    ) {
        block_on(init_actor(
            actor_id,
            move |event| {
                callback.on_event(&event.to_string());
            },
            storage_path,
        ));
    }

    pub fn iota_destroy(actor_id: &str) {
        block_on(destroy_actor(actor_id));
    }

    pub fn iota_send_message(message: &str) {
        block_on(send_actor_message(message.to_string()));
    }

    pub fn iota_listen(actor_id: &str, id: &str, event: EventType) {
        block_on(add_event_listener(actor_id, id, event));
    }

    pub fn iota_init_logger(file_name: &str) {
        let output_config = LoggerOutputConfigBuilder::new()
            .name(file_name)
            .level_filter(LevelFilter::Debug);
        init_logger(LoggerConfig::build().with_output(output_config));
    }
}
