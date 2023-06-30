use disintegrate::serde::json::Json;
use disintegrate_postgres::PgEventStore;
use serde::Deserialize;

use crate::domain::{DomainEvent, Email, PlateNumber, VehicleType};

type EventStore = PgEventStore<DomainEvent, Json<DomainEvent>>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    EventStore(#[from] disintegrate_postgres::Error),
    #[error(transparent)]
    Domain(#[from] crate::domain::Error),
}

#[derive(Clone)]
pub struct Application {
    state_store: EventStore,
}

impl Application {
    pub fn new(state_store: EventStore) -> Self {
        Self { state_store }
    }
    pub async fn register_vehicle(&self, command: RegisterVehicle) -> Result<(), Error> {
        Ok(())
    }
    pub async fn register_customer(&self, command: RegisterCustomer) -> Result<(), Error> {
        Ok(())
    }
    pub async fn start_rent(&self, command: StartRent) -> Result<(), Error> {
        Ok(())
    }
    pub async fn end_rent(&self, command: EndRent) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterVehicle {
    vehicle_id: PlateNumber,
    vehicle_type: VehicleType,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterCustomer {
    customer_id: Email,
    first_name: String,
    last_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartRent {
    customer_id: Email,
    vehicle_type: VehicleType,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EndRent {
    customer_id: Email,
    vehicle_id: PlateNumber,
}
