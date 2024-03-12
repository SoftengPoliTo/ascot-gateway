use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

use serde::{Deserialize, Serialize};

use tracing::debug;

use crate::controller::Controller;

// Device kind.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum DeviceKind {
    // Light.
    Light,
    // Fridge.
    Fridge,
}

// Route.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Route {
    // Route.
    route: String,
    // Route description.
    description: Option<String>,
}

// Data associated to a device.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DeviceData {
    // Kind.
    kind: DeviceKind,
    // Every device route.
    routes: HashMap<String, Route>,
}

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
    // Properties.
    pub(crate) properties: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Device {
    // Metadata.
    pub(crate) metadata: DeviceMetadata,
    // Data.
    pub(crate) data: DeviceData,
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

    pub(crate) fn light() -> Self {
        let mut routes = HashMap::new();
        routes.insert(
            "on".into(),
            Route {
                route: "light/on".into(),
                description: None,
            },
        );

        Self::create_device(DeviceKind::Light, routes, Controller::light())
    }

    pub(crate) fn fridge() -> Self {
        let mut routes = HashMap::new();
        routes.insert(
            "on".into(),
            Route {
                route: "fridge/on".into(),
                description: Some("Fridge is on".into()),
            },
        );

        Self::create_device(DeviceKind::Fridge, routes, Controller::fridge())
    }

    fn create_device(
        kind: DeviceKind,
        routes: HashMap<String, Route>,
        controller: Controller,
    ) -> Self {
        Self {
            metadata: DeviceMetadata {
                id: 0,
                port: 3000,
                scheme: "http".into(),
                path: "dev".into(),
                addresses: vec![DeviceAddress::new(
                    "{}://{}:{}{}".into(),
                    Ipv4Addr::LOCALHOST.into(),
                )],
                properties: HashMap::new(),
            },
            data: DeviceData { kind, routes },
            controller,
        }
    }
}

pub(crate) struct DeviceBuilder(DeviceMetadata);

impl DeviceBuilder {
    pub(crate) fn new(id: u16, port: u16, scheme: String, path: String) -> Self {
        Self(DeviceMetadata {
            id,
            port,
            scheme,
            path,
            addresses: Vec::new(),
            properties: HashMap::new(),
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

    pub(crate) fn properties(mut self, properties: HashMap<String, String>) -> Self {
        self.0.properties = properties;
        self
    }

    pub(crate) async fn build(mut self) -> Option<Device> {
        let mut device_data: Option<DeviceData> = None;

        // Try each address in order to connect to a device.
        for address in self.0.addresses.iter_mut() {
            if let Ok(response) = reqwest::get(&address.request).await {
                // When an error occurs deserializing the device information,
                // skip it.
                if let Ok(data) = response.json().await {
                    device_data = Some(data);
                    // Exit the loop as soon as data has been found
                    break;
                } else {
                    debug!("Deserialize error for address {:?}", address);
                }
            }
            address.recheable = false;
        }

        // If some device data has been found, create the device
        device_data.map(|data| {
            // Create a device data controller from a device kind
            let controller = match data.kind {
                DeviceKind::Light => Controller::light(),
                DeviceKind::Fridge => Controller::fridge(),
            };

            // Create device.
            Device {
                metadata: self.0,
                data,
                controller,
            }
        })
    }
}

/*
 * {
 *   kind: Light,
 *   routes: [
 *       "on": {
 *            route: "lights/on",
 *            description: "Turn on a light.",
 *       },
 *       "off": {
 *          route: "lights/off",
 *          description: "Turn off a light.",
 *       },
 *       "toggle": {
 *           route: "lights/toggle",
 *           description: "Toggle a light.",
 *       }
 *   ]
 *}
 */
