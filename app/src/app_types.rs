use crate::ce_converter::{try_from_event, try_to_event, Attributes};
use cloudevents::Event;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomMessageExtensions {
    something: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomMessageData {
    body: String,
    volume: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomMessage {
    #[serde(flatten)]
    attributes: Attributes,
    #[serde(flatten)]
    extensions: CustomMessageExtensions,
    data: CustomMessageData,
}

impl TryFrom<Event> for CustomMessage {
    type Error = Box<dyn Error>;
    fn try_from(ce: Event) -> Result<Self, Self::Error> {
        try_from_event(ce)
    }
}

impl TryFrom<CustomMessage> for Event {
    type Error = Box<dyn Error>;
    fn try_from(cm: CustomMessage) -> Result<Self, Self::Error> {
        try_to_event(cm)
    }
}
