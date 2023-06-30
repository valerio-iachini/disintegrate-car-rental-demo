use disintegrate::macros::Event;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Event, Serialize, Deserialize)]
pub enum DomainEvent {
    FakeEvent {
        #[id]
        id: String,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("fake error")]
    FakeError,
}

pub type PlateNumber = String;
pub type Email = String;

#[derive(Serialize, Deserialize, Debug)]
pub enum VehicleType {
    Car,
    PickUp,
    Van,
    Truck,
}

#[cfg(test)]
mod test {}
