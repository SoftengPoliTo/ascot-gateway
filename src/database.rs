use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};

use rocket_db_pools::{sqlx, sqlx::FromRow, Connection, Database};

use serde::{Deserialize, Serialize};

use crate::device::{DeviceRetriever, DeviceWidget, DeviceWidgetBuilder};

// Create a database for devices.
#[derive(Database)]
#[database("devices")]
pub(crate) struct Devices(sqlx::SqlitePool);

// Device information.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct DeviceInfo {
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
struct DeviceAddress {
    // Device address.
    address: String,
}

// Device property.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct DeviceProperty {
    // Device property key.
    key: String,
    // Device property value.
    value: String,
}

// Device route.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct DeviceRoute {
    // Identifier.
    id: u16,
    // Device route.
    route: String,
}

// Device hazard.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct DeviceHazard {
    // Device route.
    hazard: String,
}

// Device boolean input type.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct DeviceBooleanInput {
    // Device boolean name.
    name: String,
    // Device boolean value.
    value: bool,
}

// Device range input type.
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct DeviceRangeInput {
    // Device range name.
    name: String,
    // Device range type
    range_type: String,
    // Device minimum value.
    min: f64,
    // Device maximum value.
    max: f64,
    // Device step value.
    step: f64,
    // Device range value.
    value: f64,
}

// Insert a device in the database returning the associated identifier.
pub(crate) async fn insert_device(
    db: &mut Connection<Devices>,
    port: u16,
    scheme: &str,
    path: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO devices(port, scheme, path) VALUES ($1, $2, $3) RETURNING id")
        .bind(port)
        .bind(scheme)
        .bind(path)
        .fetch_one(&mut ***db)
        .await
}

// Insert device address.
pub(crate) async fn insert_device_address(
    db: &mut Connection<Devices>,
    address: String,
    id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO addresses(address, device_id) VALUES ($1, $2)")
        .bind(address)
        .bind(id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert device properties.
pub(crate) async fn insert_device_property(
    db: &mut Connection<Devices>,
    key: &str,
    value: &str,
    id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO properties(key, value, device_id) VALUES ($1, $2, $3)")
        .bind(key)
        .bind(value)
        .bind(id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert device hazard.
async fn insert_device_hazard(
    db: &mut Connection<Devices>,
    hazard: &str,
    id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO hazards(hazard, device_id) VALUES ($1, $2)")
        .bind(hazard)
        .bind(id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert device route.
async fn insert_device_route(
    db: &mut Connection<Devices>,
    route: &str,
    id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO routes(route, device_id) VALUES ($1, $2)")
        .bind(route)
        .bind(id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert boolean input for a device.
async fn insert_device_boolean_input(
    db: &mut Connection<Devices>,
    name: &str,
    value: bool,
    route_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO booleans(name, value, route_id) VALUES ($1, $2, $3)")
        .bind(name)
        .bind(value)
        .bind(route_id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert range input for a device.
async fn insert_device_range_input(
    db: &mut Connection<Devices>,
    name: &str,
    range_type: i16,
    min: f64,
    max: f64,
    step: f64,
    value: f64,
    route_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO routes(name, range_type, min, max, step, value, route_id) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(name)
        .bind(range_type)
        .bind(min)
        .bind(max)
        .bind(step)
        .bind(value)
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

    // Delete all ranges.
    sqlx::query("DELETE FROM ranges")
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

        // Delete device ranges.
        sqlx::query("DELETE FROM ranges where route_id = $1")
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
) -> Result<Vec<DeviceWidget>, sqlx::Error> {
    let devices_info: Vec<DeviceInfo> =
        sqlx::query_as("SELECT id, port, scheme, path FROM devices ORDER BY id")
            .fetch_all(&mut **db)
            .await?;

    let mut widgets = Vec::new();
    for device in devices_info.into_iter() {
        // Retrieve addresses.
        let addresses: Vec<DeviceAddress> =
            sqlx::query_as("SELECT address FROM addresses WHERE device_id = $1")
                .bind(device.id)
                .fetch_all(&mut **db)
                .await?;

        // Retrieve properties.
        let properties: Vec<DeviceProperty> =
            sqlx::query_as("SELECT key, value FROM properties WHERE device_id = $1")
                .bind(device.id)
                .fetch_all(&mut **db)
                .await?;

        // Retrieve device data.
        if let Some(device_data) =
            DeviceRetriever::new(device.id, device.port, device.scheme, device.path)
                .addresses(
                    addresses
                        .into_iter()
                        .flat_map(|a| {
                            // If an error occurs parsing an address, ignore the address
                            if let Ok(addr) = a.address.parse() {
                                Some(addr)
                            } else {
                                None
                            }
                        })
                        .collect(),
                )
                .properties(properties.into_iter().map(|v| (v.key, v.value)).collect())
                .retrieve()
                .await
        {
            // Save device routes.
            for routes in device_data.routes_configs {

            // Save device hazards.
            // Save device inputs.
            //
            // Build a device widget.
            devices.push(WidgetBuilder::first_build(
                device.id,
                device_routes,
                device_hazards,
                device_inputs,
            ));
        } else {
            // Delete a device when it is not reachable
            delete_device(&mut db, device.id).await?;
        }
    }

    Ok(devices)
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
