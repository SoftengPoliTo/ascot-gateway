use ascot_library::device::{DeviceData, DeviceKind};
use ascot_library::hazards::{CategoryData, HazardData, HazardsData};
use ascot_library::input::{Input, Inputs, InputsData};
use ascot_library::route::{RestKind, RouteConfig, RouteData, Routes};
use ascot_library::{LongString, MiniString};

use rocket::http::uri::Origin;
use rocket_db_pools::Connection;

use crate::database::controls::StateControls;
use crate::database::device::Device;
use crate::database::query::{clear_database, insert_address, insert_device};
use crate::database::{Devices, Metadata};
use crate::error::{query_error, InternalError};

fn device1() -> Device {
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

    Device {
        metadata: Metadata {
            id: 1,
            port: 8080,
            scheme: "http".into(),
            path: "here".into(),
        },
        addresses: Vec::new(),
        data: DeviceData {
            kind: DeviceKind::Light,
            main_route: MiniString::new("/light").unwrap(),
            routes,
        },
        state_controls: StateControls::default(),
    }
}

fn device2() -> Device {
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

    Device {
        metadata: Metadata {
            id: 2,
            port: 8085,
            scheme: "https".into(),
            path: "second".into(),
        },

        addresses: Vec::new(),
        data: DeviceData {
            kind: DeviceKind::Light,
            main_route: MiniString::new("/light").unwrap(),
            routes,
        },
        state_controls: StateControls::default(),
    }
}

pub(crate) async fn generate_devices_and_init_db(
    mut db: Connection<Devices>,
    uri: &Origin<'_>,
) -> Result<Vec<Device>, InternalError> {
    let mut devices = vec![device1(), device2()];

    // Clear the database.
    query_error(clear_database(&mut db), uri).await?;

    // Insert device data into the database.
    for device in devices.iter_mut() {
        let id = query_error(
            insert_device(
                &mut db,
                device.metadata.port,
                &device.metadata.scheme,
                &device.metadata.path,
            ),
            uri,
        )
        .await?;

        // Save addresses
        for address in device.addresses.iter() {
            query_error(
                insert_address(&mut db, address.address.to_string(), id),
                uri,
            )
            .await?;
        }

        query_error(device.insert_routes(&mut db), uri).await?;
    }

    Ok(devices)
}
