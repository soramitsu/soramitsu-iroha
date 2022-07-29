//! Module with telemetry layer for tracing

#![allow(clippy::module_name_repetitions)]

use std::{error::Error, fmt::Debug};

use derive_more::{Deref, DerefMut};
use serde_json::Value;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::{
    field::{Field, Visit},
    Event, Subscriber,
};

use crate::layer::{EventInspectorTrait, EventSubscriber};

/// Target for telemetry in `tracing`
pub const TELEMETRY_TARGET_PREFIX: &str = "telemetry::";
/// Target for telemetry future in `tracing`
pub const TELEMETRY_FUTURE_TARGET_PREFIX: &str = "telemetry_future::";

/// Fields for telemetry (type for efficient saving)
#[derive(Clone, Debug, PartialEq, Eq, Default, Deref, DerefMut)]
pub struct TelemetryFields(pub Vec<(&'static str, Value)>);

impl From<TelemetryFields> for Value {
    fn from(TelemetryFields(fields): TelemetryFields) -> Self {
        fields
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect()
    }
}

/// Telemetry which can be received from telemetry layer
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Telemetry {
    /// Subsystem from which telemetry was received
    pub target: &'static str,
    /// Fields which was recorded
    pub fields: TelemetryFields,
}

impl Visit for Telemetry {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.fields
            .push((field.name(), format!("{:?}", &value).into()))
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.push((field.name(), value.into()))
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.push((field.name(), value.into()))
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.push((field.name(), value.into()))
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.push((field.name(), value.into()))
    }

    fn record_error(&mut self, field: &Field, mut error: &(dyn Error + 'static)) {
        let mut vec = vec![error.to_string()];
        while let Some(inner) = error.source() {
            error = inner;
            vec.push(inner.to_string())
        }
        self.fields.push((field.name(), vec.into()))
    }
}

impl Telemetry {
    fn from_event(target: &'static str, event: &Event<'_>) -> Self {
        let fields = TelemetryFields::default();
        let mut telemetry = Self { target, fields };
        event.record(&mut telemetry);
        telemetry
    }
}

/// Telemetry layer
#[derive(Debug, Clone)]
pub struct TelemetryLayer<S: Subscriber> {
    telemetry_sender: Sender<Telemetry>,
    telemetry_future_sender: Sender<Telemetry>,
    subscriber: S,
}

impl<S: Subscriber> TelemetryLayer<S> {
    /// Create telemetry from channel sender
    pub fn from_senders(
        subscriber: S,
        telemetry_sender: Sender<Telemetry>,
        telemetry_future_sender: Sender<Telemetry>,
    ) -> impl Subscriber {
        EventSubscriber(Self {
            telemetry_sender,
            telemetry_future_sender,
            subscriber,
        })
    }

    /// Create new telemetry layer with specific channel size (via const generic)
    #[allow(clippy::new_ret_no_self)]
    pub fn new<const CHANNEL_SIZE: usize>(
        subscriber: S,
    ) -> (impl Subscriber, Receiver<Telemetry>, Receiver<Telemetry>) {
        let (sender, receiver) = mpsc::channel(CHANNEL_SIZE);
        let (sender_future, receiver_future) = mpsc::channel(CHANNEL_SIZE);
        let telemetry = Self::from_senders(subscriber, sender, sender_future);
        (telemetry, receiver, receiver_future)
    }

    /// Create new telemetry layer with specific channel size
    #[allow(clippy::new_ret_no_self)]
    pub fn from_capacity(
        subscriber: S,
        channel_size: usize,
    ) -> (impl Subscriber, Receiver<Telemetry>, Receiver<Telemetry>) {
        let (sender, receiver) = mpsc::channel(channel_size);
        let (sender_future, receiver_future) = mpsc::channel(channel_size);
        let telemetry = Self::from_senders(subscriber, sender, sender_future);
        (telemetry, receiver, receiver_future)
    }
}

impl<S: Subscriber> EventInspectorTrait for TelemetryLayer<S> {
    type Subscriber = S;

    fn inner_subscriber(&self) -> &Self::Subscriber {
        &self.subscriber
    }

    fn event(&self, event: &Event<'_>) {
        let target = event.metadata().target();
        #[allow(clippy::option_if_let_else)] // This is actually more readable.
        if let Some(telemetry_target) = target.strip_prefix(TELEMETRY_TARGET_PREFIX) {
            let _result = self
                .telemetry_sender
                .try_send(Telemetry::from_event(telemetry_target, event));
        } else if let Some(future_target) = target.strip_prefix(TELEMETRY_FUTURE_TARGET_PREFIX) {
            let _result = self
                .telemetry_future_sender
                .try_send(Telemetry::from_event(future_target, event));
        } else {
            self.subscriber.event(event)
        }
    }
}
