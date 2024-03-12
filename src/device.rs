use std::collections::HashMap;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use tracing::debug;

// Device kind.
#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) enum DeviceKind {
    // Unknown device.
    #[default]
    Unknown,
    // Light.
    Light,
}

impl From<&str> for DeviceKind {
    fn from(s: &str) -> Self {
        match s {
            "Light" => Self::Light,
            _ => Self::Unknown,
        }
    }
}

// Route.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Route {
    // Route.
    route: String,
    // Route description.
    description: String,
}

// Data associated to a device.
#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct DeviceData {
    // Kind.
    kind: DeviceKind,
    // Every device route.
    routes: Vec<Route>,
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Device {
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
    // Data.
    pub(crate) data: DeviceData,
}

impl Device {
    pub(crate) fn new(id: u16, port: u16, scheme: String, path: String) -> Self {
        Self {
            id,
            port,
            scheme,
            path,
            addresses: Vec::new(),
            properties: HashMap::new(),
            data: DeviceData::default(),
        }
    }

    pub(crate) fn addresses(mut self, addresses: Vec<IpAddr>) -> Self {
        self.addresses = addresses
            .into_iter()
            .map(|address| {
                DeviceAddress::new(
                    format!("{}://{}:{}{}", self.scheme, address, self.port, self.path),
                    address,
                )
            })
            .collect();
        self
    }

    pub(crate) fn properties(mut self, properties: HashMap<String, String>) -> Self {
        self.properties = properties;
        self
    }

    pub(crate) async fn build(mut self) -> Self {
        // Try each address in order to connect to a device.
        for address in self.addresses.iter_mut() {
            if let Ok(response) = reqwest::get(&address.request).await {
                // When an error occurs deserializing the device information,
                // skip it.
                if let Ok(data) = response.json().await {
                    self.data = data;
                } else {
                    address.recheable = false;
                    debug!("Deserialize error for address {:?}", address);
                    continue;
                }
            } else {
                address.recheable = false;
            }
        }

        self
    }

    pub(crate) fn is_recheable(&self) -> bool {
        self.addresses.iter().any(|address| address.recheable)
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
