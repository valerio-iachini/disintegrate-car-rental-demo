
use disintegrate::{decision::Error, serde::json::Json};
use disintegrate_postgres::{PgDecisionMaker, WithPgSnapshot};


use crate::domain::{DomainEvent, EndRent, RegisterCustomer, RegisterVehicle, StartRent};

pub type DecisionMaker = PgDecisionMaker<DomainEvent, Json<DomainEvent>, WithPgSnapshot>;
pub type ApplicationError = Error<crate::domain::Error>;
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
