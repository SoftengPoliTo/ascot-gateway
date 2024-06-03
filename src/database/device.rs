use std::net::IpAddr;

use ascot_library::device::DeviceData;
use ascot_library::input::InputType;

use rocket_db_pools::{sqlx, Connection};

use serde::{Deserialize, Serialize};

use tracing::debug;

use super::{Address, Devices, Metadata};

use super::controls::StateControls;
use super::query::{
    delete_device, insert_hazard, insert_main_route, insert_route, select_device_addresses,
    select_device_metadata,
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

    fn addresses(metadata: &Metadata, addresses: Vec<Address>) -> Vec<Self> {
        addresses
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
            .collect()
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Device {
    // Metadata.
    pub(crate) metadata: Metadata,
    // Addresses.
    pub(crate) addresses: Vec<DeviceAddress>,
    // Device data.
    //
    // Hazards and routes are all here.
    pub(crate) data: DeviceData,
    // Device controls with states.
    pub(crate) state_controls: StateControls,
}

impl Device {
    async fn new(metadata: Metadata, mut addresses: Vec<DeviceAddress>) -> Option<Self> {
        if let Some(data) = Self::retrieve(&mut addresses).await {
            Some(Self {
                metadata,
                addresses,
                data,
                state_controls: StateControls::default(),
            })
        } else {
            None
        }
    }

    pub(crate) fn is_recheable(&self) -> bool {
        self.addresses.iter().any(|address| address.recheable)
    }

    // Retrieve all devices for the first time.
    pub(crate) async fn search_for_devices(
        db: &mut Connection<Devices>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let devices_metadata = select_device_metadata(db).await?;

        let mut devices = Vec::new();
        for device_metadata in devices_metadata {
            // Device id.
            let device_id = device_metadata.id;

            // Retrieve addresses from database.
            let db_addresses = select_device_addresses(db, device_id).await?;

            // Construct device addresses.
            let device_addresses = DeviceAddress::addresses(&device_metadata, db_addresses);

            // If some data are retrieved, complete device creation.
            if let Some(mut device) = Device::new(device_metadata, device_addresses).await {
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
        let device_id = self.metadata.id;

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
                        self.state_controls
                            .init_slider_u64(db, route_id, input.name.as_str().to_string(), range)
                            .await?;
                    }
                    InputType::RangeF64(range) => {
                        self.state_controls.init_slider_f64(
                            db,
                            route_id,
                            input.name.as_str().to_string(),
                            range,
                        );
                    }
                    InputType::Bool(default) => {
                        self.state_controls
                            .init_checkbox(db, *default, route_id, input.name.as_str().to_string())
                            .await?
                    }
                }
            }

            self.state_controls
                .init_button(
                    db,
                    route.data.name.as_str(),
                    Self::clean_route(route.data.name.as_str()),
                    route_id,
                )
                .await?;
        }
        Ok(())
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

    async fn retrieve(addresses: &mut [DeviceAddress]) -> Option<DeviceData> {
        // Try each address in order to connect to a device.
        for address in addresses.iter_mut() {
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
