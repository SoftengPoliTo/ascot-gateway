use ascot_axum::device::DeviceData;
use ascot_axum::route::InputType;

use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};

use rocket_db_pools::{sqlx, sqlx::FromRow, Connection, Database};

use serde::{Deserialize, Serialize};

use crate::device::{Device, DeviceInfo};

// Create a database for devices.
#[derive(Database)]
#[database("devices")]
pub(crate) struct Devices(sqlx::SqlitePool);

// Device information.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Info {
    // Identifier.
    id: u16,
    // Port.
    port: u16,
    // Scheme.
    scheme: String,
    // Resource path.
    path: String,
}

// Device address.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Address {
    // Device address.
    address: String,
}

// Device property.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Property {
    // Device property key.
    key: String,
    // Device property value.
    value: String,
}

// Device route.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Route {
    // Identifier.
    id: u16,
    // Device route.
    route: String,
}

// Device hazard.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Hazard {
    // Device route.
    hazard: String,
}

// Device boolean input type.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct BooleanInput {
    // Device boolean name.
    name: String,
    // Device boolean value.
    value: bool,
}

// Device range input type for u64.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct RangeInputU64 {
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
struct RangeInputF64 {
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

// Insert a device in the database returning the associated identifier.
pub(crate) async fn insert_device(
    db: &mut Connection<Devices>,
    port: u16,
    scheme: &str,
    path: &str,
) -> Result<u16, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO devices(port, scheme, path) VALUES ($1, $2, $3) RETURNING id")
        .bind(port)
        .bind(scheme)
        .bind(path)
        .fetch_one(&mut ***db)
        .await
}

// Insert device address.
pub(crate) async fn insert_address(
    db: &mut Connection<Devices>,
    address: String,
    device_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO addresses(address, device_id) VALUES ($1, $2)")
        .bind(address)
        .bind(device_id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert device properties.
pub(crate) async fn insert_property(
    db: &mut Connection<Devices>,
    key: &str,
    value: &str,
    device_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO properties(key, value, device_id) VALUES ($1, $2, $3)")
        .bind(key)
        .bind(value)
        .bind(device_id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert device hazard.
async fn insert_hazard(
    db: &mut Connection<Devices>,
    hazard: &str,
    device_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO hazards(hazard, device_id) VALUES ($1, $2)")
        .bind(hazard)
        .bind(device_id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert device route.
async fn insert_route(
    db: &mut Connection<Devices>,
    route: &str,
    device_id: u16,
) -> Result<u16, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO routes(route, device_id) VALUES ($1, $2) RETURNING id")
        .bind(route)
        .bind(device_id)
        .fetch_one(&mut ***db)
        .await
}

// Insert boolean input for a device.
async fn insert_boolean_input(
    db: &mut Connection<Devices>,
    name: &str,
    default: bool,
    value: bool,
    route_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO booleans(name, default, value, route_id) VALUES ($1, $2, $3, $4)")
        .bind(name)
        .bind(default)
        .bind(value)
        .bind(route_id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert range input for u64.
async fn insert_rangeu64_input(
    db: &mut Connection<Devices>,
    range: RangeInputU64,
    route_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO rangesu64(name, min, max, step, default, value, route_id) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(range.name)
    .bind(range.min as i64)
    .bind(range.max as i64)
    .bind(range.step as i64)
    .bind(range.default as i64)
    .bind(range.value as i64)
    .bind(route_id)
    .execute(&mut ***db)
    .await?;
    Ok(())
}

// Insert range input for f64.
async fn insert_rangef64_input(
    db: &mut Connection<Devices>,
    range: RangeInputF64,
    route_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO rangesf64(name, min, max, step, default, value, route_id) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(range.name)
    .bind(range.min)
    .bind(range.max)
    .bind(range.step)
    .bind(range.default)
    .bind(range.value)
    .bind(route_id)
    .execute(&mut ***db)
    .await?;
    Ok(())
}

// Delete all devices data.
pub(crate) async fn delete_all_devices(db: &mut Connection<Devices>) -> Result<(), sqlx::Error> {
    // Delete all booleans.
    sqlx::query("DELETE FROM booleans")
        .execute(&mut ***db)
        .await?;

    // Delete all u64 ranges.
    sqlx::query("DELETE FROM rangesu64")
        .execute(&mut ***db)
        .await?;

    // Delete all f64 ranges.
    sqlx::query("DELETE FROM rangesf64")
        .execute(&mut ***db)
        .await?;

    // Delete all routes.
    sqlx::query("DELETE FROM routes")
        .execute(&mut ***db)
        .await?;

    // Delete all properties.
    sqlx::query("DELETE FROM properties")
        .execute(&mut ***db)
        .await?;

    // Delete all addresses.
    sqlx::query("DELETE FROM addresses")
        .execute(&mut ***db)
        .await?;

    // Delete all hazards.
    sqlx::query("DELETE FROM hazards")
        .execute(&mut ***db)
        .await?;

    // Delete all devices.
    sqlx::query("DELETE FROM devices")
        .execute(&mut ***db)
        .await?;

    Ok(())
}

// Delete all devices data.
async fn delete_device(db: &mut Connection<Devices>, id: u16) -> Result<(), sqlx::Error> {
    #[derive(FromRow, Serialize, Deserialize)]
    struct RouteId(u16);

    // Get routes id associated with the device.
    let routes_id: Vec<RouteId> = sqlx::query_as("SELECT id FROM routes WHERE device_id = $1")
        .bind(id)
        .fetch_all(&mut ***db)
        .await?;

    // Delete inputs
    for route_id in routes_id {
        // Delete device booleans.
        sqlx::query("DELETE FROM booleans where route_id = $1")
            .bind(route_id.0)
            .execute(&mut ***db)
            .await?;

        // Delete device u64 ranges.
        sqlx::query("DELETE FROM rangesu64 where route_id = $1")
            .bind(route_id.0)
            .execute(&mut ***db)
            .await?;

        // Delete device f64 ranges.
        sqlx::query("DELETE FROM rangesf64 where route_id = $1")
            .bind(route_id.0)
            .execute(&mut ***db)
            .await?;
    }

    // Delete device routes.
    sqlx::query("DELETE FROM routes WHERE device_id = $1")
        .bind(id)
        .execute(&mut ***db)
        .await?;

    // Delete device properties.
    sqlx::query("DELETE FROM properties WHERE device_id = $1")
        .bind(id)
        .execute(&mut ***db)
        .await?;

    // Delete device addresses.
    sqlx::query("DELETE FROM addresses WHERE device_id = $1")
        .bind(id)
        .execute(&mut ***db)
        .await?;

    // Delete device hazards.
    sqlx::query("DELETE FROM hazards WHERE device_id = $1")
        .bind(id)
        .execute(&mut ***db)
        .await?;

    // Delete device.
    sqlx::query("DELETE FROM devices WHERE id = $1")
        .bind(id)
        .execute(&mut ***db)
        .await?;

    Ok(())
}

// Retrieve all devices for the first time.
pub(crate) async fn first_time_devices<'a>(
    mut db: Connection<Devices>,
) -> Result<Vec<Device>, sqlx::Error> {
    let devices_info: Vec<Info> =
        sqlx::query_as("SELECT id, port, scheme, path FROM devices ORDER BY id")
            .fetch_all(&mut **db)
            .await?;

    let mut devices = Vec::new();
    for device in devices_info.into_iter() {
        // Retrieve addresses.
        let addresses: Vec<Address> =
            sqlx::query_as("SELECT address FROM addresses WHERE device_id = $1")
                .bind(device.id)
                .fetch_all(&mut **db)
                .await?;

        // Retrieve device data.
        if let Some(device_info) =
            DeviceInfo::new(device.id, device.port, device.scheme, device.path)
                .addresses(
                    addresses
                        .into_iter()
                        .filter_map(|a| a.address.parse().ok())
                        .collect(),
                )
                .retrieve()
                .await
        {
            // Insert routes.
            insert_routes(&mut db, device.id, &device_info).await?;
            // Build a new device for the first time.
            //devices.push(DeviceBuilder::first_time(device.id, device_info));
        } else {
            // Delete a device when it is not reachable
            delete_device(&mut db, device.id).await?;
        }
    }

    Ok(devices)
}

// Insert routes.
async fn insert_routes<'a>(
    db: &mut Connection<Devices>,
    device_id: u16,
    device_info: &DeviceData<'a>,
) -> Result<(), sqlx::Error> {
    for route in device_info.routes_configs.iter() {
        // Save device routes into database.
        let route_id = insert_route(db, &route.data.name, device_id).await?;

        // Save device hazards into database.
        for hazard in route.hazards.iter() {
            insert_hazard(db, &hazard.name, device_id).await?;
        }

        // If a route does not have an input and it is a PUT REST,
        // the input is a boolean.
        if route.data.inputs.is_empty() {
            insert_boolean_input(db, &route.data.name, false, false, route_id).await?;
            continue;
        }

        // Save device inputs into database.
        for input in route.data.inputs.iter() {
            match &input.datatype {
                InputType::RangeU64(range) => {
                    let range = RangeInputU64 {
                        name: input.name.to_string(),
                        min: range.minimum,
                        max: range.maximum,
                        step: range.step,
                        default: range.default,
                        value: range.default,
                    };
                    insert_rangeu64_input(db, range, route_id).await?;
                }
                InputType::RangeF64(range) => {
                    let range = RangeInputF64 {
                        name: input.name.to_string(),
                        min: range.minimum,
                        max: range.maximum,
                        step: range.step,
                        default: range.default,
                        value: range.default,
                    };
                    insert_rangef64_input(db, range, route_id).await?;
                }
                InputType::Bool(default) => {
                    insert_boolean_input(db, &input.name, *default, *default, route_id).await?
                }
            }
        }
    }
    Ok(())
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
