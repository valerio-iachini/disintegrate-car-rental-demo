use disintegrate::serde::json::Json;
use disintegrate::StateStore;
use disintegrate_postgres::PgEventStore;
use serde::Deserialize;

use crate::domain::{
    CustomerRegistration, DomainEvent, Email, PlateNumber, VehicleRegistration, VehicleRental,
    VehicleType,
};

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
        let vehicle_registration = self
            .state_store
            .hydrate(VehicleRegistration::new(command.vehicle_id))
            .await?;

        let event = dbg!(vehicle_registration.add(command.vehicle_type)?);
        self.state_store
            .save(&vehicle_registration, vec![event])
            .await?;

        Ok(())
    }

    pub async fn register_customer(&self, command: RegisterCustomer) -> Result<(), Error> {
        let customer_registration = self
            .state_store
            .hydrate(CustomerRegistration::new(command.customer_id))
            .await?;

        let event = dbg!(customer_registration.register(command.first_name, command.last_name)?);
        self.state_store
            .save(&customer_registration, vec![event])
            .await?;

        Ok(())
    }

    pub async fn start_rent(&self, command: StartRent) -> Result<(), Error> {
        let vehicle_rental = self
            .state_store
            .hydrate(VehicleRental::new(
                command.customer_id,
                command.vehicle_type,
            ))
            .await?;

        let event = dbg!(vehicle_rental.rent()?);
        self.state_store.save(&vehicle_rental, vec![event]).await?;

        Ok(())
    }

    pub async fn end_rent(&self, command: EndRent) -> Result<(), Error> {
        let vehicle_rental = self
            .state_store
            .hydrate(VehicleRental::new(
                command.customer_id,
                command.vehicle_type,
            ))
            .await?;

        let event = dbg!(vehicle_rental.end()?);
        self.state_store.save(&vehicle_rental, vec![event]).await?;

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
    vehicle_type: VehicleType,
}
