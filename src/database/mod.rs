pub(crate) mod device;
pub(crate) mod query;

use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};

use rocket_db_pools::{sqlx, sqlx::FromRow, Database};

use serde::{Deserialize, Serialize};

// Create a database for devices.
#[derive(Database)]
#[database("devices")]
pub(crate) struct Devices(sqlx::SqlitePool);

// Device metadata.
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub(super) struct Metadata {
    // Identifier.
    id: u16,
    // Port.
    port: u16,
    // Scheme.
    scheme: String,
    // Resource path.
    path: String,
}

impl Metadata {
    pub(super) fn fake1() -> Self {
        Self {
            id: 0,
            port: 8080,
            scheme: "http".into(),
            path: "here".into(),
        }
    }

    pub(super) fn fake2() -> Self {
        Self {
            id: 0,
            port: 8080,
            scheme: "http".into(),
            path: "here".into(),
        }
    }
}

// Device address.
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub(super) struct Address {
    // Device address.
    address: String,
}

// Device property.
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub(super) struct Property {
    // Device property key.
    key: String,
    // Device property value.
    value: String,
}

// Device route.
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub(super) struct Route {
    // Identifier.
    id: u16,
    // Device route.
    route: String,
}

// Device hazard.
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub(super) struct Hazard {
    // Device route.
    hazard: String,
}

// Device boolean input type.
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub(super) struct BooleanInput {
    // Device boolean name.
    name: String,
    // Device boolean value.
    value: bool,
}

// Device range input type for u64.
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub(super) struct RangeInputU64 {
    // Input name.
    name: String,
    // Minimum value.
    min: u64,
    // Maximum value.
    max: u64,
    // Step value.
    step: u64,
    // Default value.
    default: u64,
    // Current value.
    value: u64,
}

// Device range input type for f64.
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub(super) struct RangeInputF64 {
    // Input name.
    name: String,
    // Minimum value.
    min: f64,
    // Maximum value.
    max: f64,
    // Step value.
    step: f64,
    // Default value.
    default: f64,
    // Current value.
    value: f64,
}

// Runs database migrations scripts.
//
// All database tables are created during this phase.
async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Devices::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

// Create a middle layer to define the database during server creation.
pub(crate) fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(Devices::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
    })
}
