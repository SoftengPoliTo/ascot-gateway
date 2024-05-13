use std::net::IpAddr;

use ascot_axum::device::DeviceData;
use ascot_axum::route::InputType;

use rocket_db_pools::{sqlx, Connection};

use serde::{Deserialize, Serialize};

use tracing::debug;

use crate::controller::Controller;

use super::{Address, Devices, Metadata, RangeInputF64, RangeInputU64};

use super::query::{
    delete_device, insert_boolean_input, insert_hazard, insert_rangef64_input,
    insert_rangeu64_input, insert_route, select_device_addresses, select_device_metadata,
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
pub(crate) struct Device<'a> {
    // Metadata.
    metadata: Metadata,
    // Addresses.
    pub(crate) addresses: Vec<DeviceAddress>,
    // Device data.
    pub(crate) data: Option<DeviceData<'a>>,
}

impl<'a> Device<'a> {
    pub(crate) fn is_recheable(&self) -> bool {
        self.addresses.iter().any(|address| address.recheable)
    }

    // Retrieve all devices for the first time.
    pub(crate) async fn search_for_devices(
        db: &mut Connection<Devices>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let devices_info = select_device_metadata(db).await?;

        let mut devices = Vec::new();
        for device_info in devices_info {
            // Retrieve addresses.
            let addresses = select_device_addresses(db, device_info.id).await?;

            // Create a device.
            let mut device = Device::new(device_info, addresses);

            // Retrieve device data.
            device.retrieve().await;

            // Retrieve device data.
            if let Some(ref device_data) = device.data {
                // Insert routes.
                Self::insert_routes(db, device.metadata.id, &device_data).await?;
                // Build a new device for the first time.
                devices.push(device);
            } else {
                // Delete a device when it is not reachable
                delete_device(db, device.metadata.id).await?;
            }
        }

        Ok(devices)
    }

    fn new(metadata: Metadata, addresses: Vec<Address>) -> Self {
        let addresses = addresses
            .into_iter()
            .filter_map(|a| {
                a.address.parse().ok().map(|address| {
                    DeviceAddress::new(
                        format!(
                            "{}://{}:{}{}",
                            metadata.scheme, address, metadata.port, metadata.path
                        ),
                        address,
                    )
                })
            })
            .collect();

        Self {
            metadata,
            addresses,
            data: None,
        }
    }

    async fn retrieve(&mut self) {
        // Try each address in order to connect to a device.
        for address in self.addresses.iter_mut() {
            if let Ok(response) = reqwest::get(&address.request).await {
                // When an error occurs deserializing the device information,
                // skip it.
                if let Ok(data) = response.json().await {
                    self.data = Some(data);
                    // Exit the loop as soon as data has been found
                    break;
                } else {
                    debug!("Deserialize error for address {:?}", address);
                }
            }
            address.recheable = false;
        }
    }

    // Insert routes.
    async fn insert_routes(
        db: &mut Connection<Devices>,
        device_id: u16,
        device_data: &DeviceData<'a>,
    ) -> Result<(), sqlx::Error> {
        for route in device_data.routes_configs.iter() {
            // Save device routes into database.
            let route_id = insert_route(db, &route.data.name, device_id).await?;

            // Save device hazards into database.
            for hazard in route.hazards.iter() {
                insert_hazard(db, hazard.id, device_id).await?;
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
}
