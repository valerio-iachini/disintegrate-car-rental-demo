#![allow(clippy::enum_variant_names)]
use std::{collections::HashSet, fmt::Display};

use chrono::{DateTime, Utc};
use disintegrate::{
    Event, IdentifierType, IdentifierValue, IntoIdentifierValue, StateMutate, StateQuery,
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

            RentEvent::VehicleRented {
                vehicle_id,
                ..
            } => {
                self.available_vehicles.remove(&vehicle_id);
            }

            RentEvent::VehicleReturned {
                vehicle_id,
                ..
            } => {
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
