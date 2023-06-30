use std::{collections::HashSet, fmt::Display};

use chrono::{DateTime, Utc};
use disintegrate::{macros::Event, query, State};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Event, Serialize, Deserialize)]
#[group(CustomerEvent, [CustomerRegistered])]
#[group(VehicleEvent, [VehicleAdded])]
#[group(RentEvent, [VehicleAdded, CustomerRegistered, VehicleRented, VehicleReturned])]
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

#[derive(Clone)]
pub struct CustomerRegistration {
    customer_id: Email,
    registered: bool,
}

impl State for CustomerRegistration {
    type Event = CustomerEvent;

    fn query(&self) -> disintegrate::StreamQuery<Self::Event> {
        query!(CustomerEvent, customer_id == self.customer_id)
    }

    fn mutate(&mut self, event: Self::Event) {
        match event {
            CustomerEvent::CustomerRegistered { .. } => self.registered = true,
        }
    }
}

impl CustomerRegistration {
    pub fn new(customer_id: String) -> Self {
        Self {
            customer_id,
            registered: false,
        }
    }
    pub fn register(&self, first_name: String, last_name: String) -> Result<CustomerEvent, Error> {
        if self.registered {
            return Err(Error::AlreadyRegisteredCustomer);
        }
        Ok(CustomerEvent::CustomerRegistered {
            customer_id: self.customer_id.clone(),
            first_name,
            last_name,
        })
    }
}

#[derive(Clone)]
pub struct VehicleRegistration {
    vehicle_id: PlateNumber,
    registered: bool,
}

impl State for VehicleRegistration {
    type Event = VehicleEvent;

    fn query(&self) -> disintegrate::StreamQuery<Self::Event> {
        query!(VehicleEvent, vehicle_id == self.vehicle_id)
    }

    fn mutate(&mut self, event: Self::Event) {
        match event {
            VehicleEvent::VehicleAdded { .. } => self.registered = true,
        }
    }
}

impl VehicleRegistration {
    pub fn new(vehicle_id: PlateNumber) -> Self {
        Self {
            vehicle_id,
            registered: false,
        }
    }
    pub fn add(&self, vehicle_type: VehicleType) -> Result<VehicleEvent, Error> {
        if self.registered {
            return Err(Error::AlreadyRegisteredVehicle);
        }
        Ok(VehicleEvent::VehicleAdded {
            vehicle_id: self.vehicle_id.clone(),
            vehicle_type,
        })
    }
}

#[derive(Clone)]
pub struct VehicleRental {
    vehicle_type: VehicleType,
    customer_id: Email,
    rented_vehicle_id: Option<PlateNumber>,
    customer_registered: bool,
    available_vehicles: HashSet<PlateNumber>,
}

impl State for VehicleRental {
    type Event = RentEvent;

    fn query(&self) -> disintegrate::StreamQuery<Self::Event> {
        query!(RentEvent, (vehicle_type == self.vehicle_type) or (customer_id == self.customer_id) )
    }

    fn mutate(&mut self, event: Self::Event) {
        match event {
            RentEvent::CustomerRegistered { .. } => self.customer_registered = true,
            RentEvent::VehicleAdded { vehicle_id, .. } => {
                self.available_vehicles.insert(vehicle_id);
            }

            RentEvent::VehicleRented {
                vehicle_id,
                customer_id,
                ..
            } => {
                if self.customer_id == customer_id {
                    self.rented_vehicle_id = Some(vehicle_id.clone());
                }
                self.available_vehicles.remove(&vehicle_id);
            }

            RentEvent::VehicleReturned {
                vehicle_id,
                customer_id,
                ..
            } => {
                if self.customer_id == customer_id {
                    self.rented_vehicle_id = None;
                }
                self.available_vehicles.insert(vehicle_id);
            }
        };
    }
}

impl VehicleRental {
    pub fn new(customer_id: Email, vehicle_type: VehicleType) -> Self {
        Self {
            customer_id,
            vehicle_type,
            customer_registered: false,
            rented_vehicle_id: None,
            available_vehicles: HashSet::new(),
        }
    }
    pub fn rent(&self) -> Result<RentEvent, Error> {
        if !self.customer_registered {
            return Err(Error::CustomerNotFound);
        }

        let Some(vehicle) = self.available_vehicles.iter().last() else { return Err(Error::NoAvailableVehicles)};

        if self.rented_vehicle_id.is_some() {
            return Err(Error::RentalInProgress);
        }

        Ok(RentEvent::VehicleRented {
            customer_id: self.customer_id.to_owned(),
            vehicle_type: self.vehicle_type.to_owned(),
            vehicle_id: vehicle.to_owned(),
            start_date: Utc::now(),
        })
    }
    pub fn end(&self) -> Result<RentEvent, Error> {
        if !self.customer_registered {
            return Err(Error::CustomerNotFound);
        }

        if let Some(rented_vehicle_id) = self.rented_vehicle_id.as_ref() {
            Ok(RentEvent::VehicleReturned {
                customer_id: self.customer_id.to_owned(),
                vehicle_type: self.vehicle_type.to_owned(),
                returned_date: Utc::now(),
                vehicle_id: rented_vehicle_id.to_owned(),
            })
        } else {
            return Err(Error::RentalNotFound);
        }
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn it_should_not_register_customer_twice() {
        disintegrate::TestHarness::given(
            CustomerRegistration::new("customer".to_string()),
            [CustomerEvent::CustomerRegistered {
                customer_id: "customer".to_string(),
                first_name: "".to_string(),
                last_name: "".to_string(),
            }],
        )
        .when(|s| s.register("Pippo".to_owned(), "Pluto".to_owned()))
        .then_err(Error::AlreadyRegisteredCustomer);
    }
}
