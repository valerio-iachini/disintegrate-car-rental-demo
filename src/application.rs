use chrono::Utc;
use disintegrate::{decision::Error, serde::json::Json, Decision};
use disintegrate_postgres::{PgDecisionMaker, WithPgSnapshot};
use serde::Deserialize;

use crate::domain::{
    CustomerRegistration, CustomerRentalStatus, DomainEvent, Email, Error as DomainError,
    PlateNumber, VehicleAvailability, VehicleRegistration, VehicleType,
};

pub type DecisionMaker = PgDecisionMaker<DomainEvent, Json<DomainEvent>, WithPgSnapshot>;
pub type ApplicationError = Error<DomainError>;
pub type ApplicationResult = Result<(), ApplicationError>;

#[derive(Clone)]
pub struct Application {
    decision_maker: DecisionMaker,
}

impl Application {
    pub fn new(decision_maker: DecisionMaker) -> Self {
        Self { decision_maker }
    }
    pub async fn register_vehicle(&self, command: RegisterVehicle) -> ApplicationResult {
        self.decision_maker.make(command).await?;

        Ok(())
    }

    pub async fn register_customer(&self, command: RegisterCustomer) -> ApplicationResult {
        self.decision_maker.make(command).await?;
        Ok(())
    }

    pub async fn start_rent(&self, command: StartRent) -> ApplicationResult {
        self.decision_maker.make(command).await?;

        Ok(())
    }

    pub async fn end_rent(&self, command: EndRent) -> ApplicationResult {
        self.decision_maker.make(command).await?;

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterVehicle {
    vehicle_id: PlateNumber,
    vehicle_type: VehicleType,
}

impl Decision for RegisterVehicle {
    type Event = DomainEvent;

    type StateQuery = VehicleRegistration;

    type Error = DomainError;

    fn state_query(&self) -> Self::StateQuery {
        VehicleRegistration::new(self.vehicle_id.clone())
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.registered {
            return Err(DomainError::AlreadyRegisteredVehicle);
        }
        Ok(vec![DomainEvent::VehicleAdded {
            vehicle_id: self.vehicle_id.clone(),
            vehicle_type: self.vehicle_type.clone(),
        }])
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterCustomer {
    customer_id: Email,
    first_name: String,
    last_name: String,
}

impl Decision for RegisterCustomer {
    type Event = DomainEvent;

    type StateQuery = CustomerRegistration;

    type Error = DomainError;

    fn state_query(&self) -> Self::StateQuery {
        CustomerRegistration::new(self.customer_id.clone())
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.registered {
            return Err(DomainError::AlreadyRegisteredCustomer);
        }
        Ok(vec![DomainEvent::CustomerRegistered {
            customer_id: self.customer_id.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
        }])
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartRent {
    customer_id: Email,
    vehicle_type: VehicleType,
}

impl Decision for StartRent {
    type Event = DomainEvent;

    type StateQuery = (
        CustomerRegistration,
        CustomerRentalStatus,
        VehicleAvailability,
    );

    type Error = DomainError;

    fn state_query(&self) -> Self::StateQuery {
        (
            CustomerRegistration::new(self.customer_id.clone()),
            CustomerRentalStatus::new(self.customer_id.clone()),
            VehicleAvailability::new(self.vehicle_type.clone()),
        )
    }

    fn process(
        &self,
        (customer_registration, customer_rental_status, vehicle_availability): &Self::StateQuery,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        if !customer_registration.registered {
            return Err(DomainError::CustomerNotFound);
        }

        let Some(vehicle) = vehicle_availability.available_vehicles.iter().last() else {
            return Err(DomainError::NoAvailableVehicles);
        };

        if customer_rental_status.rented_vehicle_id.is_some() {
            return Err(DomainError::RentalInProgress);
        }

        Ok(vec![DomainEvent::VehicleRented {
            customer_id: self.customer_id.to_owned(),
            vehicle_type: self.vehicle_type.to_owned(),
            vehicle_id: vehicle.to_owned(),
            start_date: Utc::now(),
        }])
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EndRent {
    customer_id: Email,
}

impl Decision for EndRent {
    type Event = DomainEvent;

    type StateQuery = CustomerRentalStatus;

    type Error = DomainError;

    fn state_query(&self) -> Self::StateQuery {
        CustomerRentalStatus::new(self.customer_id.clone())
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if let Some(rented_vehicle_id) = state.rented_vehicle_id.as_ref() {
            Ok(vec![DomainEvent::VehicleReturned {
                customer_id: self.customer_id.to_owned(),
                vehicle_type: state.rented_vehicle_type.as_ref().unwrap().clone(),
                returned_date: Utc::now(),
                vehicle_id: rented_vehicle_id.to_owned(),
            }])
        } else {
            Err(DomainError::RentalNotFound)
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn it_should_not_register_customer_twice() {
        disintegrate::TestHarness::given([DomainEvent::CustomerRegistered {
            customer_id: "customer".to_string(),
            first_name: "Bob".to_string(),
            last_name: "Solo".to_string(),
        }])
        .when(RegisterCustomer {
            customer_id: "customer".to_string(),
            first_name: "Bob".to_string(),
            last_name: "Solo".to_string(),
        })
        .then_err(DomainError::AlreadyRegisteredCustomer);
    }
}
