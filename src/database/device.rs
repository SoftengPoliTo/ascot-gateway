use std::borrow::Cow;
use std::net::IpAddr;

use ascot_library::device::DeviceData;
use ascot_library::route::InputType;

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
pub(crate) struct DeviceInfo {
    // Metadata.
    pub(crate) metadata: Metadata,
    // Addresses.
    pub(crate) addresses: Vec<DeviceAddress>,
}

impl DeviceInfo {
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
        }
    }

    async fn retrieve<'a>(&mut self) -> Option<DeviceData<'a>> {
        // Try each address in order to connect to a device.
        for address in self.addresses.iter_mut() {
            if let Ok(response) = reqwest::get(&address.request).await {
                // When an error occurs deserializing the device information,
                // skip it.
                if let Ok(data) = response.json().await {
                    return Some(data);
                } else {
                    debug!("Deserialize error for address {:?}", address);
                }
            }
            address.recheable = false;
        }
        None
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Device<'a> {
    // Device info.
    pub(crate) info: DeviceInfo,
    // Device data.
    //
    // Hazards are all here.
    pub(crate) data: DeviceData<'a>,
}

impl<'a> Device<'a> {
    fn new(info: DeviceInfo, data: DeviceData<'a>) -> Self {
        Self { info, data }
    }

    pub(crate) fn is_recheable(&self) -> bool {
        self.info.addresses.iter().any(|address| address.recheable)
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

            // Define device information.
            let mut device_info = DeviceInfo::new(device_info, addresses);

            // If some data has been retrieved, complete device creation.
            if let Some(device_data) = device_info.retrieve().await {
                // Insert routes.
                let hazards =
                    Self::insert_routes(db, device_info.metadata.id, &device_data).await?;

                // Create device.
                let device = Device::new(device_info, device_data);

                // Save device.
                devices.push(device);
            } else {
                // Delete a device when it is not reachable
                delete_device(db, device_info.metadata.id).await?;
            }
        }

        Ok(devices)
    }

    // Insert routes.
    async fn insert_routes(
        db: &mut Connection<Devices>,
        device_id: u16,
        device_data: &DeviceData<'a>,
    ) -> Result<Vec<Inputs>, sqlx::Error> {
        let mut inputs = Vec::new();
        for route in device_data.routes_configs.iter() {
            // Save device routes into database.
            let route_id = insert_route(db, &route.data.name, device_id).await?;

            for hazard in route.hazards.iter() {
                // Save device hazards into database.
                insert_hazard(db, hazard.id, device_id).await?;
            }

            // Insert route as a boolean value.
            insert_boolean_input(db, &route.data.name, false, false, route_id).await?;

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
        Ok(inputs)
    }
}
