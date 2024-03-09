#![allow(clippy::enum_variant_names)]
use std::{collections::HashSet, fmt::Display};

use chrono::{DateTime, Utc};
use disintegrate::{
    Decision, Event, IdentifierType, IdentifierValue, IntoIdentifierValue, StateMutate, StateQuery,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Event, Serialize, Deserialize)]
#[group(CustomerEvent, [CustomerRegistered])]
#[group(VehicleEvent, [VehicleAdded])]
#[group(RentEvent, [VehicleAdded, VehicleRented, VehicleReturned])]
pub enum DomainEvent {
    CustomerRegistered {
        #[id]
        customer_id: Email,
        first_name: String,
        last_name: String,
    },
    VehicleAdded {
        #[id]
        vehicle_id: PlateNumber,
        #[id]
        vehicle_type: VehicleType,
    },
    VehicleRented {
        #[id]
        customer_id: Email,
        #[id]
        vehicle_id: PlateNumber,
        #[id]
        vehicle_type: VehicleType,
        start_date: DateTime<Utc>,
    },
    VehicleReturned {
        #[id]
        customer_id: Email,
        #[id]
        vehicle_id: PlateNumber,
        #[id]
        vehicle_type: VehicleType,
        returned_date: DateTime<Utc>,
    },
}

#[derive(Debug, StateQuery, Clone, Serialize, Deserialize)]
#[state_query(CustomerEvent)]
pub struct CustomerRegistration {
    #[id]
    pub(crate) customer_id: Email,
    pub(crate) registered: bool,
}

impl CustomerRegistration {
    pub fn new(customer_id: String) -> Self {
        Self {
            customer_id,
            registered: false,
        }
    }
}

impl StateMutate for CustomerRegistration {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            CustomerEvent::CustomerRegistered { .. } => self.registered = true,
        }
    }
}

#[derive(Debug, StateQuery, Clone, Serialize, Deserialize)]
#[state_query(VehicleEvent)]
pub struct VehicleRegistration {
    #[id]
    pub(crate) vehicle_id: PlateNumber,
    pub(crate) registered: bool,
}

impl VehicleRegistration {
    pub fn new(vehicle_id: PlateNumber) -> Self {
        Self {
            vehicle_id,
            registered: false,
        }
    }
}

impl StateMutate for VehicleRegistration {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            VehicleEvent::VehicleAdded { .. } => self.registered = true,
        }
    }
}

#[derive(Debug, StateQuery, Clone, Serialize, Deserialize)]
#[state_query(RentEvent)]
pub struct VehicleAvailability {
    #[id]
    pub(crate) vehicle_type: VehicleType,
    pub(crate) available_vehicles: HashSet<PlateNumber>,
}

impl VehicleAvailability {
    pub fn new(vehicle_type: VehicleType) -> Self {
        Self {
            vehicle_type,
            available_vehicles: HashSet::new(),
        }
    }
}

impl StateMutate for VehicleAvailability {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            RentEvent::VehicleAdded { vehicle_id, .. } => {
                self.available_vehicles.insert(vehicle_id);
            }

            RentEvent::VehicleRented { vehicle_id, .. } => {
                self.available_vehicles.remove(&vehicle_id);
            }

            RentEvent::VehicleReturned { vehicle_id, .. } => {
                self.available_vehicles.insert(vehicle_id);
            }
        };
    }
}

#[derive(Debug, StateQuery, Clone, Serialize, Deserialize)]
#[state_query(RentEvent)]
pub struct CustomerRentalStatus {
    #[id]
    pub(crate) customer_id: Email,
    pub(crate) rented_vehicle_type: Option<VehicleType>,
    pub(crate) rented_vehicle_id: Option<PlateNumber>,
}

impl CustomerRentalStatus {
    pub fn new(customer_id: Email) -> Self {
        Self {
            customer_id,
            rented_vehicle_type: None,
            rented_vehicle_id: None,
        }
    }
}

impl StateMutate for CustomerRentalStatus {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            RentEvent::VehicleAdded { .. } => {}

            RentEvent::VehicleRented {
                vehicle_id,
                vehicle_type,
                ..
            } => {
                self.rented_vehicle_id = Some(vehicle_id);
                self.rented_vehicle_type = Some(vehicle_type);
            }

            RentEvent::VehicleReturned { .. } => {
                self.rented_vehicle_id = None;
                self.rented_vehicle_type = None;
            }
        };
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("Already Registered Vehicle")]
    AlreadyRegisteredVehicle,
    #[error("Already Registered Customer")]
    AlreadyRegisteredCustomer,
    #[error("No Available Vehicles")]
    NoAvailableVehicles,
    #[error("Rental In Progress")]
    RentalInProgress,
    #[error("Customer Not Found")]
    CustomerNotFound,
    #[error("Rental Not Found")]
    RentalNotFound,
}

pub type PlateNumber = String;
pub type Email = String;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum VehicleType {
    Car,
    PickUp,
    Van,
    Truck,
}

impl IntoIdentifierValue for VehicleType {
    const TYPE: disintegrate::IdentifierType = IdentifierType::String;

    fn into_identifier_value(self) -> disintegrate::IdentifierValue {
        IdentifierValue::String(self.to_string())
    }
}

impl Display for VehicleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VehicleType::Car => write!(f, "car"),
            VehicleType::PickUp => write!(f, "pick_up"),
            VehicleType::Van => write!(f, "van"),
            VehicleType::Truck => write!(f, "truck"),
        }
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

    type Error = Error;

    fn state_query(&self) -> Self::StateQuery {
        VehicleRegistration::new(self.vehicle_id.clone())
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.registered {
            return Err(Error::AlreadyRegisteredVehicle);
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

    type Error = Error;

    fn state_query(&self) -> Self::StateQuery {
        CustomerRegistration::new(self.customer_id.clone())
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.registered {
            return Err(Error::AlreadyRegisteredCustomer);
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

    type Error = Error;

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
            return Err(Error::CustomerNotFound);
        }

        let Some(vehicle) = vehicle_availability.available_vehicles.iter().last() else {
            return Err(Error::NoAvailableVehicles);
        };

        if customer_rental_status.rented_vehicle_id.is_some() {
            return Err(Error::RentalInProgress);
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

    type Error = Error;

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
            Err(Error::RentalNotFound)
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
        .then_err(Error::AlreadyRegisteredCustomer);
    }
}
