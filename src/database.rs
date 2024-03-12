use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};

use rocket_db_pools::{sqlx, sqlx::FromRow, Connection, Database};

use serde::{Deserialize, Serialize};

use crate::device::Device;

const INSERT_DEVICE: &str = "INSERT INTO devices(port, scheme, path)
     VALUES ($1, $2, $3) RETURNING id";

const INSERT_DEVICE_ADDRESS: &str = "INSERT INTO addresses(address, id)
     VALUES ($1, $2)";

const INSERT_DEVICE_PROPERTY: &str = "INSERT INTO properties(key, value, id)
     VALUES ($1, $2, $3)";

const ALL_DEVICES: &str = "SELECT id, port, scheme, path FROM devices ORDER BY id";

const ADDRESSES: &str = "SELECT address FROM addresses WHERE id = $1";

const PROPERTIES: &str = "SELECT key, value FROM properties WHERE id = $1";

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

// Insert a device in the database returning the associated identifier.
pub(crate) async fn insert_device(
    db: &mut Connection<Devices>,
    port: u16,
    scheme: &str,
    path: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(INSERT_DEVICE)
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
    sqlx::query(INSERT_DEVICE_ADDRESS)
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
    sqlx::query(INSERT_DEVICE_PROPERTY)
        .bind(key)
        .bind(value)
        .bind(id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Retrieve all devices.
pub(crate) async fn all_devices(mut db: Connection<Devices>) -> Result<Vec<Device>, sqlx::Error> {
    let devices_info: Vec<DeviceInfo> = sqlx::query_as(ALL_DEVICES).fetch_all(&mut **db).await?;

    let mut devices = Vec::new();
    for device in devices_info.into_iter() {
        let addresses: Vec<DeviceAddress> = sqlx::query_as(ADDRESSES)
            .bind(device.id)
            .fetch_all(&mut **db)
            .await?;
        let properties: Vec<DeviceProperty> = sqlx::query_as(PROPERTIES)
            .bind(device.id)
            .fetch_all(&mut **db)
            .await?;
        devices.push(
            Device::new(device.id, device.port, device.scheme, device.path)
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
                .build()
                .await,
        );
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
