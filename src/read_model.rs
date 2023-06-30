use crate::domain::{DomainEvent, Email, PlateNumber, VehicleType};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use disintegrate::{query, EventListener, PersistedEvent, StreamQuery};
use sqlx::{FromRow, PgPool};

#[derive(Clone)]
pub struct Repository {
    pool: PgPool,
}

impl Repository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn vehicles(&self) -> Result<Vec<Vehicle>, sqlx::Error> {
        sqlx::query_as::<_, Vehicle>("SELECT * FROM vehicle")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn customers(&self) -> Result<Vec<Customer>, sqlx::Error> {
        sqlx::query_as::<_, Customer>("SELECT * FROM customer")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn rents(&self) -> Result<Vec<Rent>, sqlx::Error> {
        sqlx::query_as::<_, Rent>("SELECT * FROM rent")
            .fetch_all(&self.pool)
            .await
    }
}

#[derive(FromRow)]
pub struct Vehicle {
    pub vehicle_id: String,
    pub vehicle_type: String,
}

#[derive(FromRow)]
pub struct Customer {
    pub customer_id: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(FromRow)]
pub struct Rent {
    pub customer_id: String,
    pub vehicle_id: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

pub struct ReadModelProjection {
    query: StreamQuery<DomainEvent>,
    pool: PgPool,
}

impl ReadModelProjection {
    pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
        sqlx::query(
            r#"
                CREATE TABLE IF NOT EXISTS vehicle (
                    vehicle_id TEXT PRIMARY KEY,
                    vehicle_type TEXT
                )"#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
                CREATE TABLE IF NOT EXISTS customer (
                    customer_id TEXT PRIMARY KEY,
                    fist_name TEXT,
                    last_name TEXT
           )"#,
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rent (
                customer_Id TEXT,
                vehicle_id TEXT,
                star_date timestamptz, 
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
            _ => {}
        }
        Ok(())
    }
}
