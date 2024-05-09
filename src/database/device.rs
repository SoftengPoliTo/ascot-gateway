use std::net::IpAddr;

use ascot_axum::device::DeviceData;
use ascot_axum::route::InputType;

use rocket_db_pools::{sqlx, sqlx::FromRow, Connection};

use serde::{Deserialize, Serialize};

use tracing::debug;

use crate::controller::Controller;

use super::{Devices, RangeInputF64, RangeInputU64};

use super::query::{
    insert_boolean_input, insert_hazard, insert_rangef64_input, insert_rangeu64_input,
    insert_route, select_device_addresses, select_device_info,
};

// Device addresses.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DeviceAddress {
    // Whether the address is reachable.
    recheable: bool,
    // Address.
    address: IpAddr,
    // Request.
    request: String,
}

impl DeviceAddress {
    fn new(request: String, address: IpAddr) -> Self {
        Self {
            recheable: true,
            address,
            request,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct DeviceMetadata {
    // Identifier.
    pub(crate) id: u16,
    // Port.
    pub(crate) port: u16,
    // Scheme.
    pub(crate) scheme: String,
    // Resource path.
    pub(crate) path: String,
    // Addresses.
    pub(crate) addresses: Vec<DeviceAddress>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Device {
    // Metadata.
    pub(crate) metadata: DeviceMetadata,
    // Device data controller.
    pub(crate) controller: Controller,
}

impl Device {
    pub(crate) fn is_recheable(&self) -> bool {
        self.metadata
            .addresses
            .iter()
            .any(|address| address.recheable)
    }
}

pub(crate) struct DeviceInfo(DeviceMetadata);

impl DeviceInfo {
    pub(crate) fn new(id: u16, port: u16, scheme: String, path: String) -> Self {
        Self(DeviceMetadata {
            id,
            port,
            scheme,
            path,
            addresses: Vec::new(),
        })
    }

    pub(crate) fn addresses(mut self, addresses: Vec<IpAddr>) -> Self {
        self.0.addresses = addresses
            .into_iter()
            .map(|address| {
                DeviceAddress::new(
                    format!(
                        "{}://{}:{}{}",
                        self.0.scheme, address, self.0.port, self.0.path
                    ),
                    address,
                )
            })
            .collect();
        self
    }

    pub(crate) async fn retrieve<'a>(mut self) -> Option<DeviceData<'a>> {
        let mut device_info: Option<DeviceData> = None;

        // Try each address in order to connect to a device.
        for address in self.0.addresses.iter_mut() {
            if let Ok(response) = reqwest::get(&address.request).await {
                // When an error occurs deserializing the device information,
                // skip it.
                if let Ok(data) = response.json().await {
                    device_info = Some(data);
                    // Exit the loop as soon as data has been found
                    break;
                } else {
                    debug!("Deserialize error for address {:?}", address);
                }
            }
            address.recheable = false;
        }

        device_info
    }
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

// Retrieve all devices for the first time.
pub(crate) async fn first_time_devices<'a>(
    mut db: Connection<Devices>,
) -> Result<Vec<Device>, sqlx::Error> {
    let devices_info = select_device_info(&mut db).await?;

    let mut devices = Vec::new();
    for device in devices_info {
        // Retrieve addresses.
        let addresses = select_device_addresses(&mut db, device.id).await?;

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
