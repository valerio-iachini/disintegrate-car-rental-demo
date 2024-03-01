mod application;
mod domain;
mod read_model;

use std::{
    fmt::{self},
    time::Duration,
};

use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    post,
    web::{Data, Json},
    App, HttpResponse, HttpServer,
};
use application::{Application, ApplicationError};
use disintegrate_postgres::{PgEventListener, PgEventListenerConfig, PgEventStore};
use domain::DomainEvent;
use sqlx::{postgres::PgConnectOptions, PgPool};
use tokio::signal;

use crate::application::{EndRent, RegisterCustomer, RegisterVehicle, StartRent};

type EventStore = PgEventStore<DomainEvent, disintegrate::serde::json::Json<DomainEvent>>;

#[derive(Debug)]
struct CarRentalResponseError(ApplicationError);
impl From<ApplicationError> for CarRentalResponseError {
    fn from(value: ApplicationError) -> Self {
        Self(value)
    }
}
impl fmt::Display for CarRentalResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().unwrap();

    let connect_options = PgConnectOptions::new();
    let pool = PgPool::connect_with(connect_options).await?;

    let serde = disintegrate::serde::json::Json::<DomainEvent>::default();

    let event_store = PgEventStore::new(pool.clone(), serde).await?;

    let decision_maker =
        disintegrate_postgres::decision_maker_with_snapshot(event_store.clone(), 10).await?;

    let application = Application::new(decision_maker);

    tokio::try_join!(http_server(application), event_listener(pool, event_store))?;
    Ok(())
}

async fn http_server(app: Application) -> anyhow::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app.clone()))
            .service(register_vehicle)
            .service(register_customer)
            .service(rent_start)
            .service(rent_end)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}

#[post("/vehicle/register")]
async fn register_vehicle(
    app: Data<Application>,
    data: Json<RegisterVehicle>,
) -> Result<&'static str, CarRentalResponseError> {
    dbg!(&data);
    app.register_vehicle(data.into_inner()).await?;
    Ok("success!")
}

#[post("/customer/register")]
async fn register_customer(
    app: Data<Application>,
    data: Json<RegisterCustomer>,
) -> Result<&'static str, CarRentalResponseError> {
    dbg!(&data);
    app.register_customer(data.into_inner()).await?;
    Ok("success!")
}

#[post("/rent/start")]
async fn rent_start(
    app: Data<Application>,
    data: Json<StartRent>,
) -> Result<&'static str, CarRentalResponseError> {
    dbg!(&data);
    app.start_rent(data.into_inner()).await?;
    Ok("success!")
}

#[post("/rent/end")]
async fn rent_end(
    app: Data<Application>,
    data: Json<EndRent>,
) -> Result<&'static str, CarRentalResponseError> {
    dbg!(&data);
    app.end_rent(data.into_inner()).await?;
    Ok("success!")
}

impl error::ResponseError for CarRentalResponseError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self.0 {
            disintegrate::decision::Error::Domain(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn event_listener(pool: sqlx::PgPool, event_store: EventStore) -> anyhow::Result<()> {
    PgEventListener::builder(event_store)
        .register_listener(
            read_model::ReadModelProjection::new(pool.clone())
                .await
                .unwrap(),
            PgEventListenerConfig::poller(Duration::from_millis(50)),
        )
        .start_with_shutdown(shutdown())
        .await
        .map_err(|e| anyhow::anyhow!("event listener exited with error: {}", e))
}

async fn shutdown() {
    signal::ctrl_c().await.expect("failed to listen for event");
}
