use std::net::IpAddr;

use ascot_library::device::DeviceData;
use ascot_library::input::InputType;

use rocket_db_pools::{sqlx, Connection};

use serde::{Deserialize, Serialize};

use tracing::debug;

use crate::controls::{Button, CheckBox, Slider};

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

    async fn retrieve<'a>(&mut self) -> Option<DeviceData> {
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
pub(crate) struct Device {
    // Device info.
    pub(crate) info: DeviceInfo,
    // Device data.
    //
    // Hazards are all here.
    pub(crate) data: DeviceData,
    // Sliders u64.
    pub(crate) sliders_u64: Vec<Slider<u64>>,
    // Sliders f64.
    pub(crate) sliders_f64: Vec<Slider<f64>>,
    // Checkboxes.
    pub(crate) checkboxes: Vec<CheckBox>,
    // Buttons.
    pub(crate) buttons: Vec<Button>,
}

impl Device {
    fn new(info: DeviceInfo, data: DeviceData) -> Self {
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
    pub(crate) async fn insert_routes(
        mut self,
        db: &mut Connection<Devices>,
    ) -> Result<Self, sqlx::Error> {
        let device_id = self.info.metadata.id;

        // Insert main route.
        insert_main_route(db, self.data.main_route.as_str(), device_id).await?;

        /*for route in self.data.routes.iter() {
            // Save device routes into database.
            let route_id = insert_route(db, route.data.name.as_str(), device_id).await?;

            for hazard in route.hazards.iter() {
                // Save device hazards into database.
                insert_hazard(db, hazard.id, device_id).await?;
            }

            // Insert route as a boolean value into the database.
            insert_boolean_input(db, route.data.name.as_str(), false, false, route_id).await?;

            // Insert button for a route.
            self.buttons.push(Button::init(
                route_id,
                Self::clean_route(route.data.name.as_str()),
            ));

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
                        // Insert u64 slider.
                        self.sliders_u64.push(Slider::<u64>::new(
                            route_id,
                            input.name.as_str().to_string(),
                            range.minimum,
                            range.maximum,
                            range.step,
                            range.default,
                        ));
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
                        // Insert f64 slider.
                        self.sliders_f64.push(Slider::<f64>::new(
                            route_id,
                            input.name.as_str().to_string(),
                            range.minimum,
                            range.maximum,
                            range.step,
                            range.default,
                        ));
                    }
                    InputType::Bool(default) => {
                        insert_boolean_input(db, input.name.as_str(), *default, *default, route_id)
                            .await?;
                        self.checkboxes
                            .push(CheckBox::init(route_id, input.name.as_str().to_string()));
                    }
                }
            }
        }*/
        Ok(self)
    }

    // Insert routes.
    pub(crate) async fn insert_routes2(
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

            // Insert route as a boolean value into the database.
            insert_boolean_input(db, route.data.name.as_str(), false, false, route_id).await?;

            // Insert button for a route.
            self.buttons.push(Button::init(
                route_id,
                Self::clean_route(route.data.name.as_str()),
            ));

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
                        // Insert u64 slider.
                        self.sliders_u64.push(Slider::<u64>::new(
                            route_id,
                            input.name.as_str().to_string(),
                            range.minimum,
                            range.maximum,
                            range.step,
                            range.default,
                        ));
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
                        // Insert f64 slider.
                        self.sliders_f64.push(Slider::<f64>::new(
                            route_id,
                            input.name.as_str().to_string(),
                            range.minimum,
                            range.maximum,
                            range.step,
                            range.default,
                        ));
                    }
                    InputType::Bool(default) => {
                        insert_boolean_input(db, input.name.as_str(), *default, *default, route_id)
                            .await?;
                        self.checkboxes
                            .push(CheckBox::init(route_id, input.name.as_str().to_string()));
                    }
                }
            }
        }
        Ok(())
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
        use ascot_library::hazards::{CategoryData, HazardData, HazardsData};
        use ascot_library::input::{Input, Inputs, InputsData};
        use ascot_library::route::{RestKind, RouteConfig, RouteData, Routes};
        use ascot_library::{LongString, MiniString};

        let mut routes = Routes::init();

        let mut inputs = Inputs::init();
        inputs.add(Input::rangef64("brightness", (0., 20., 0.1, 0.)));
        inputs.add(Input::boolean("save-energy", false));

        let mut hazards = HazardsData::init();
        hazards.add(HazardData {
            id: 0,
            name: MiniString::new("Fire Hazard").unwrap(),
            description: LongString::new("An Hazard fire").unwrap(),
            category: CategoryData {
                name: MiniString::new("Safety").unwrap(),
                description: LongString::new("A safety category").unwrap(),
            },
        });

        hazards.add(HazardData {
            id: 1,
            name: MiniString::new("Energy Consumption").unwrap(),
            description: LongString::new("Consuming energy").unwrap(),
            category: CategoryData {
                name: MiniString::new("Financial").unwrap(),
                description: LongString::new("Reduce Energy").unwrap(),
            },
        });

        let light_on = RouteConfig {
            rest_kind: RestKind::Put,
            hazards,
            data: RouteData {
                name: MiniString::new("/on/<brightness>/<save-energy>").unwrap(),
                description: Some(LongString::new("Light on").unwrap()),
                stateless: false,
                inputs: InputsData::from_inputs(&inputs).unwrap(),
            },
        };

        let light_off = RouteConfig {
            rest_kind: RestKind::Put,
            hazards: HazardsData::init(),
            data: RouteData {
                name: MiniString::new("/off").unwrap(),
                description: Some(LongString::new("Light off").unwrap()),
                stateless: false,
                inputs: InputsData::init(),
            },
        };

        let toggle = RouteConfig {
            rest_kind: RestKind::Put,
            hazards: HazardsData::init(),
            data: RouteData {
                name: MiniString::new("/toggle").unwrap(),
                description: None,
                stateless: false,
                inputs: InputsData::init(),
            },
        };

        routes.add(light_on);
        routes.add(light_off);
        routes.add(toggle);

        Self {
            info: DeviceInfo {
                metadata: Metadata {
                    id: 1,
                    port: 8080,
                    scheme: "http".into(),
                    path: "here".into(),
                },
                addresses: Vec::new(),
            },
            data: DeviceData {
                kind: DeviceKind::Light,
                main_route: MiniString::new("/light").unwrap(),
                routes,
            },
            buttons: Vec::new(),
            sliders_u64: Vec::new(),
            sliders_f64: Vec::new(),
            checkboxes: Vec::new(),
            /*buttons: vec![
                Button::init(0, Self::clean_route("/on/<brightness>/<save-energy>")),
                Button::init(1, Self::clean_route("/off")),
                Button::init(2, Self::clean_route("/toggle")),
            ],
            sliders_u64: Vec::new(),
            sliders_f64: vec![Slider::<f64>::new(0, "brightness".into(), 0., 20., 0.1, 5.)],
            checkboxes: vec![CheckBox::init(0, "save-energy".into())],*/
        }
    }

    pub(crate) fn fake_device2() -> Self {
        use ascot_library::device::DeviceKind;
        use ascot_library::hazards::{CategoryData, HazardData, HazardsData};
        use ascot_library::input::{Input, Inputs, InputsData};
        use ascot_library::route::{RestKind, RouteConfig, RouteData, Routes};
        use ascot_library::{LongString, MiniString};

        let mut routes = Routes::init();

        let mut inputs = Inputs::init();
        inputs.add(Input::rangef64("brightness", (0., 20., 0.1, 0.)));
        inputs.add(Input::boolean("save-energy", false));

        let mut hazards = HazardsData::init();
        hazards.add(HazardData {
            id: 0,
            name: MiniString::new("Fire Hazard").unwrap(),
            description: LongString::new("An Hazard fire").unwrap(),
            category: CategoryData {
                name: MiniString::new("Safety").unwrap(),
                description: LongString::new("A safety category").unwrap(),
            },
        });

        hazards.add(HazardData {
            id: 1,
            name: MiniString::new("Energy Consumption").unwrap(),
            description: LongString::new("Consuming energy").unwrap(),
            category: CategoryData {
                name: MiniString::new("Financial").unwrap(),
                description: LongString::new("Reduce Energy").unwrap(),
            },
        });

        let light_on = RouteConfig {
            rest_kind: RestKind::Put,
            hazards,
            data: RouteData {
                name: MiniString::new("/on/<brightness>/<save-energy>").unwrap(),
                description: Some(LongString::new("Light on").unwrap()),
                stateless: false,
                inputs: InputsData::from_inputs(&inputs).unwrap(),
            },
        };

        let light_off = RouteConfig {
            rest_kind: RestKind::Put,
            hazards: HazardsData::init(),
            data: RouteData {
                name: MiniString::new("/off").unwrap(),
                description: Some(LongString::new("Light off").unwrap()),
                stateless: false,
                inputs: InputsData::init(),
            },
        };

        let mut inputs2 = Inputs::init();
        inputs2.add(Input::rangeu64("dimmer", (0, 15, 1, 2)));

        let toggle = RouteConfig {
            rest_kind: RestKind::Put,
            hazards: HazardsData::init(),
            data: RouteData {
                name: MiniString::new("/toggle").unwrap(),
                description: None,
                stateless: false,
                inputs: InputsData::from_inputs(&inputs2).unwrap(),
            },
        };

        routes.add(light_on);
        routes.add(light_off);
        routes.add(toggle);

        Self {
            info: DeviceInfo {
                metadata: Metadata {
                    id: 2,
                    port: 8085,
                    scheme: "https".into(),
                    path: "second".into(),
                },

                addresses: Vec::new(),
            },
            data: DeviceData {
                kind: DeviceKind::Light,
                main_route: MiniString::new("/light").unwrap(),
                routes,
            },
            buttons: Vec::new(),
            sliders_u64: Vec::new(),
            sliders_f64: Vec::new(),
            checkboxes: Vec::new(),
            /*buttons: vec![
                Button::init(0, Self::clean_route("/on/<brightness>/<save-energy>")),
                Button::init(1, Self::clean_route("/off")),
                Button::init(2, Self::clean_route("/toggle/:dimmer")),
            ],
            sliders_u64: vec![Slider::<u64>::new(2, "dimmer".into(), 0, 15, 1, 2)],
            sliders_f64: vec![Slider::<f64>::new(0, "brightness".into(), 0., 20., 0.1, 5.)],
            checkboxes: vec![CheckBox::init(0, "save-energy".into())],*/
        }
    }
}
