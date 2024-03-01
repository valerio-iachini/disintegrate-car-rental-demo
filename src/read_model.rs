use crate::domain::DomainEvent;
use async_trait::async_trait;

use disintegrate::{query, EventListener, PersistedEvent, StreamQuery};
use sqlx::{PgPool};

pub struct ReadModelProjection {
    query: StreamQuery<DomainEvent>,
    pool: PgPool,
}

impl ReadModelProjection {
    pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS vehicle (
                vehicle_id TEXT PRIMARY KEY,
                vehicle_type TEXT
            )"#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS customer (
                customer_id TEXT PRIMARY KEY,
                first_name TEXT,
                last_name TEXT
            )"#,
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS rent (
                customer_id TEXT,
                vehicle_id TEXT,
                start_date timestamptz, 
                end_date timestamptz NULL,
                PRIMARY KEY(customer_id, vehicle_id)
            )"#,
        )
        .execute(&pool)
        .await?;
        Ok(Self {
            query: query(None),
            pool,
        })
    }
}

#[async_trait]
impl EventListener<DomainEvent> for ReadModelProjection {
    type Error = sqlx::Error;
    fn id(&self) -> &'static str {
        "drive_me_crazy_rentals"
    }

    fn query(&self) -> &StreamQuery<DomainEvent> {
        &self.query
    }

    async fn handle(&self, event: PersistedEvent<DomainEvent>) -> Result<(), Self::Error> {
        match event.into_inner() {
            DomainEvent::CustomerRegistered {
                customer_id,
                first_name,
                last_name,
            } =>  sqlx::query(
                    "INSERT INTO customer (customer_id, first_name, last_name) VALUES($1, $2, $3)",
                )
                .bind(customer_id)
                .bind(first_name)
                .bind(last_name)
                .execute(&self.pool)
                .await
                .unwrap(),
            DomainEvent::VehicleAdded {
                vehicle_id,
                vehicle_type,
            } => sqlx::query(
                    "INSERT INTO vehicle (vehicle_id, vehicle_type) VALUES($1, $2)",
                )
                .bind(vehicle_id)
                .bind(vehicle_type.to_string())
                .execute(&self.pool)
                .await
                .unwrap(),
            DomainEvent::VehicleRented {
                customer_id,
                vehicle_id,
                vehicle_type: _,
                start_date,
            } => sqlx::query(
                    "INSERT INTO rent (customer_id, vehicle_id, start_date) VALUES($1, $2, $3)",
                )
                .bind(customer_id)
                .bind(vehicle_id)
                .bind(start_date)
                .execute(&self.pool)
                .await
                .unwrap(),
            DomainEvent::VehicleReturned {
                customer_id,
                vehicle_id,
                vehicle_type: _,
                returned_date,
            } => sqlx::query(
                    "UPDATE rent SET end_date = $3 where customer_id = $1 and vehicle_id = $2 and end_date is null",
                )
                .bind(customer_id)
                .bind(vehicle_id)
                .bind(returned_date)
                .execute(&self.pool)
                .await
                .unwrap(),
        };
        Ok(())
    }
}
