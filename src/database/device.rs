use std::borrow::Cow;
use std::net::IpAddr;

use ascot_library::device::DeviceData;
use ascot_library::route::InputType;

use rocket_db_pools::{sqlx, Connection};

use serde::{Deserialize, Serialize};

use tracing::debug;

use crate::controls::{Button, CheckBox, Slider};

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
    // Sliders u64.
    pub(crate) sliders_u64: Vec<Slider<u64>>,
    // Sliders f64.
    pub(crate) sliders_f64: Vec<Slider<f64>>,
    // Checkboxes.
    pub(crate) checkboxes: Vec<CheckBox>,
    // Buttons.
    pub(crate) buttons: Vec<Button>,
}

impl<'a> Device<'a> {
    fn new(info: DeviceInfo, data: DeviceData<'a>) -> Self {
        Self {
            info,
            data,
            sliders_u64: Vec::new(),
            sliders_f64: Vec::new(),
            checkboxes: Vec::new(),
            buttons: Vec::new(),
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
            // Retrieve addresses.
            let addresses = select_device_addresses(db, device_info.id).await?;

            // Define device information.
            let mut device_info = DeviceInfo::new(device_info, addresses);

            // If some data has been retrieved, complete device creation.
            if let Some(device_data) = device_info.retrieve().await {
                // Create device.
                let device = Device::new(device_info, device_data)
                    .insert_routes(db)
                    .await?;

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
    async fn insert_routes(mut self, db: &mut Connection<Devices>) -> Result<Self, sqlx::Error> {
        let device_id = self.info.metadata.id;
        for route in self.data.routes_configs.iter() {
            // Save device routes into database.
            let route_id = insert_route(db, &route.data.name, device_id).await?;

            for hazard in route.hazards.iter() {
                // Save device hazards into database.
                insert_hazard(db, hazard.id, device_id).await?;
            }

            // Insert route as a boolean value into the database.
            insert_boolean_input(db, &route.data.name, false, false, route_id).await?;

            // Insert button for a route.
            self.buttons
                .push(Button::init(Self::clean_route(&route.data.name)));

            // Save device inputs into database.
            for input in route.data.inputs.iter() {
                match &input.datatype {
                    InputType::RangeU64(range) => {
                        let range_db = RangeInputU64 {
                            name: input.name.to_string(),
                            min: range.minimum,
                            max: range.maximum,
                            step: range.step,
                            default: range.default,
                            value: range.default,
                        };
                        insert_rangeu64_input(db, range_db, route_id).await?;
                        // Insert u64 slider.
                        self.sliders_u64.push(Slider::<u64>::new(
                            input.name.to_string(),
                            range.minimum,
                            range.maximum,
                            range.step,
                            range.default,
                        ));
                    }
                    InputType::RangeF64(range) => {
                        let range_db = RangeInputF64 {
                            name: input.name.to_string(),
                            min: range.minimum,
                            max: range.maximum,
                            step: range.step,
                            default: range.default,
                            value: range.default,
                        };
                        insert_rangef64_input(db, range_db, route_id).await?;
                        // Insert f64 slider.
                        self.sliders_f64.push(Slider::<f64>::new(
                            input.name.to_string(),
                            range.minimum,
                            range.maximum,
                            range.step,
                            range.default,
                        ));
                    }
                    InputType::Bool(default) => {
                        insert_boolean_input(db, &input.name, *default, *default, route_id).await?;
                        self.checkboxes.push(CheckBox::init(input.name.to_string()));
                    }
                }
            }
        }
        Ok(self)
    }

    // Clean route.
    fn clean_route(route: &str) -> String {
        let no_prefix = match route.strip_prefix("/") {
            Some(no_prefix) => no_prefix,
            None => return "<unknown>".into(),
        };

        if let Some(name) = no_prefix.split_once("/") {
            name.0
        } else {
            no_prefix
        }
        .to_string()
    }

    pub(crate) fn fake_device1() -> Self {
        use ascot_library::device::DeviceKind;
        use ascot_library::hazards::{CategoryData, HazardData};
        use ascot_library::route::{Input, RestKind, RouteConfig, RouteData};
        use heapless::{FnvIndexSet, Vec};

        let mut routes_configs: Vec<RouteConfig, 16> = Vec::new();

        let mut inputs: FnvIndexSet<Input<'a>, 16> = FnvIndexSet::new();
        let _ = inputs.insert(Input::rangef64("brightness", (0., 20., 0.1, 0.)));
        let _ = inputs.insert(Input::boolean("save-energy", false));

        let mut hazards: FnvIndexSet<HazardData, 16> = FnvIndexSet::new();
        let _ = hazards.insert(HazardData {
            id: 0,
            name: "Fire Hazard".into(),
            description: "An Hazard fire".into(),
            category: CategoryData {
                name: "Safety".into(),
                description: "A safety category".into(),
            },
        });

        let _ = hazards.insert(HazardData {
            id: 1,
            name: "Energy Consumption".into(),
            description: "Consuming energy".into(),
            category: CategoryData {
                name: "Financial".into(),
                description: "Reduce Energy".into(),
            },
        });

        let light_on = RouteConfig {
            rest_kind: RestKind::Put,
            hazards,
            data: RouteData {
                name: "/on/<brightness>/<save-energy>".into(),
                description: Some("Light on".into()),
                stateless: false,
                inputs,
            },
        };

        let light_off = RouteConfig {
            rest_kind: RestKind::Put,
            hazards: FnvIndexSet::new(),
            data: RouteData {
                name: "/off".into(),
                description: Some("Light off".into()),
                stateless: false,
                inputs: FnvIndexSet::new(),
            },
        };

        let toggle = RouteConfig {
            rest_kind: RestKind::Put,
            hazards: FnvIndexSet::new(),
            data: RouteData {
                name: "/toggle".into(),
                description: None,
                stateless: false,
                inputs: FnvIndexSet::new(),
            },
        };

        let _ = routes_configs.push(light_on);
        let _ = routes_configs.push(light_off);
        let _ = routes_configs.push(toggle);

        Self {
            info: DeviceInfo {
                metadata: Metadata::fake1(),
                addresses: std::vec::Vec::new(),
            },
            data: DeviceData {
                kind: DeviceKind::Light,
                main_route: "/light".into(),
                routes_configs,
            },
            buttons: vec![
                Button::init(Self::clean_route("/on/<brightness>/<save-energy>")),
                Button::init(Self::clean_route("/off")),
                Button::init(Self::clean_route("/toggle")),
            ],
            sliders_f64: vec![Slider::<f64>::new("brightness".into(), 0., 20., 0.1, 5.)],
            sliders_u64: std::vec::Vec::new(),
            checkboxes: vec![CheckBox::init("save-energy".into())],
        }
    }

    pub(crate) fn fake_device2() -> Self {
        use ascot_library::device::DeviceKind;
        use ascot_library::hazards::{CategoryData, HazardData};
        use ascot_library::route::{Input, RestKind, RouteConfig, RouteData};
        use heapless::{FnvIndexSet, Vec};

        let mut routes_configs: Vec<RouteConfig, 16> = Vec::new();

        let mut inputs: FnvIndexSet<Input<'a>, 16> = FnvIndexSet::new();
        let _ = inputs.insert(Input::rangef64("brightness", (0., 20., 0.1, 0.)));
        let _ = inputs.insert(Input::boolean("save-energy", false));

        let mut hazards: FnvIndexSet<HazardData, 16> = FnvIndexSet::new();
        let _ = hazards.insert(HazardData {
            id: 0,
            name: "Fire Hazard".into(),
            description: "An Hazard fire".into(),
            category: CategoryData {
                name: "Safety".into(),
                description: "A safety category".into(),
            },
        });

        let _ = hazards.insert(HazardData {
            id: 1,
            name: "Energy Consumption".into(),
            description: "Consuming energy".into(),
            category: CategoryData {
                name: "Financial".into(),
                description: "Reduce Energy".into(),
            },
        });

        let light_on = RouteConfig {
            rest_kind: RestKind::Put,
            hazards,
            data: RouteData {
                name: "/on/<brightness>/<save-energy>".into(),
                description: Some("Light on".into()),
                stateless: false,
                inputs,
            },
        };

        let light_off = RouteConfig {
            rest_kind: RestKind::Put,
            hazards: FnvIndexSet::new(),
            data: RouteData {
                name: "/off".into(),
                description: Some("Light off".into()),
                stateless: false,
                inputs: FnvIndexSet::new(),
            },
        };

        let mut inputs2: FnvIndexSet<Input<'a>, 16> = FnvIndexSet::new();
        let _ = inputs2.insert(Input::rangeu64("dimmer", (0, 15, 1, 2)));

        let toggle = RouteConfig {
            rest_kind: RestKind::Put,
            hazards: FnvIndexSet::new(),
            data: RouteData {
                name: "/toggle".into(),
                description: None,
                stateless: false,
                inputs: inputs2,
            },
        };

        let _ = routes_configs.push(light_on);
        let _ = routes_configs.push(light_off);
        let _ = routes_configs.push(toggle);

        Self {
            info: DeviceInfo {
                metadata: Metadata::fake1(),
                addresses: std::vec::Vec::new(),
            },
            data: DeviceData {
                kind: DeviceKind::Light,
                main_route: "/light".into(),
                routes_configs,
            },
            buttons: vec![
                Button::init(Self::clean_route("/on/<brightness>/<save-energy>")),
                Button::init(Self::clean_route("/off")),
                Button::init(Self::clean_route("/toggle/:dimmer")),
            ],
            sliders_f64: vec![Slider::<f64>::new("brightness".into(), 0., 20., 0.1, 5.)],
            sliders_u64: vec![Slider::<u64>::new("dimmer".into(), 0, 15, 1, 2)],
            checkboxes: vec![CheckBox::init("save-energy".into())],
        }
    }
}
