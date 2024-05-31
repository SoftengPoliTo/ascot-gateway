use std::net::IpAddr;

use ascot_library::device::DeviceData;
use ascot_library::input::InputType;

use rocket_db_pools::{sqlx, Connection};

use serde::{Deserialize, Serialize};

use tracing::debug;

use crate::controls::Controls;

use super::{Address, Devices, Metadata, RangeInputF64, RangeInputU64};

use super::query::{
    delete_device, insert_boolean_input, insert_hazard, insert_main_route, insert_rangef64_input,
    insert_rangeu64_input, insert_route, select_device_addresses, select_device_metadata,
};

// Device addresses.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DeviceAddress {
    // Whether the address is reachable.
    recheable: bool,
    // Address.
    pub(crate) address: IpAddr,
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

    async fn retrieve(mut self) -> Option<(Self, DeviceData)> {
        // Try each address in order to connect to a device.
        for address in self.addresses.iter_mut() {
            if let Ok(response) = reqwest::get(&address.request).await {
                // When an error occurs deserializing the device information,
                // skip it.
                if let Ok(data) = response.json().await {
                    return Some((self, data));
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
pub(crate) struct Device {
    // Device info.
    pub(crate) info: DeviceInfo,
    // Device data.
    //
    // Hazards are all here.
    pub(crate) data: DeviceData,
    // Device controls.
    pub(crate) controls: Controls,
}

impl Device {
    fn new(info: DeviceInfo, data: DeviceData) -> Self {
        Self {
            info,
            data,
            controls: Controls::default(),
        }
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
            // Device id.
            let device_id = device_info.id;

            // Retrieve device addresses.
            let addresses = select_device_addresses(db, device_id).await?;

            // If some data are retrieved, complete device creation.
            if let Some((device_info, device_data)) =
                DeviceInfo::new(device_info, addresses).retrieve().await
            {
                // Create device.
                let mut device = Device::new(device_info, device_data);

                // Insert routes.
                device.insert_routes(db).await?;

                // Save device.
                devices.push(device);
            } else {
                // Delete a device when it is not reachable
                delete_device(db, device_id).await?;
            }
        }

        Ok(devices)
    }

    // Insert routes.
    pub(crate) async fn insert_routes(
        &mut self,
        db: &mut Connection<Devices>,
    ) -> Result<(), sqlx::Error> {
        let device_id = self.info.metadata.id;

        // Insert main route.
        insert_main_route(db, self.data.main_route.as_str(), device_id).await?;

        for route in self.data.routes.iter() {
            // Save device routes into database.
            let route_id = insert_route(db, route.data.name.as_str(), device_id).await?;

            for hazard in route.hazards.iter() {
                // Save device hazards into database.
                insert_hazard(db, hazard.id, device_id).await?;
            }

            // Save device inputs into database.
            for input in route.data.inputs.iter() {
                match &input.datatype {
                    InputType::RangeU64(range) => {
                        let range_db = RangeInputU64 {
                            name: input.name.as_str().to_string(),
                            min: range.minimum,
                            max: range.maximum,
                            step: range.step,
                            default: range.default,
                            value: range.default,
                        };
                        insert_rangeu64_input(db, range_db, route_id).await?;
                        self.controls.init_sliders_u64(
                            route_id,
                            input.name.as_str().to_string(),
                            range,
                        );
                    }
                    InputType::RangeF64(range) => {
                        let range_db = RangeInputF64 {
                            name: input.name.as_str().to_string(),
                            min: range.minimum,
                            max: range.maximum,
                            step: range.step,
                            default: range.default,
                            value: range.default,
                        };
                        insert_rangef64_input(db, range_db, route_id).await?;
                        self.controls.init_sliders_f64(
                            route_id,
                            input.name.as_str().to_string(),
                            range,
                        );
                    }
                    InputType::Bool(default) => {
                        insert_boolean_input(db, input.name.as_str(), *default, *default, route_id)
                            .await?;
                        self.controls
                            .init_checkbox(route_id, input.name.as_str().to_string());
                        Self::checkbox(&mut self.controls, db);
                    }
                }
            }

            // Insert route as a boolean value into the database.
            insert_boolean_input(db, route.data.name.as_str(), false, false, route_id).await?;

            // Initialize a button for a route.
            self.controls
                .init_button(route_id, Self::clean_route(route.data.name.as_str()));
        }
        Ok(())
    }

    // Insert checkbox.
    #[inline]
    fn checkbox(val: &mut Controls) {
        let a = 5;
    }

    // Clean route.
    #[inline]
    fn clean_route(route: &str) -> String {
        route
            .strip_prefix("/")
            .map_or("<unknown route>", |no_prefix| {
                no_prefix
                    .split_once("/")
                    .map(|name| name.0)
                    .unwrap_or(no_prefix)
            })
            .into()
    }
}
