use std::collections::HashMap;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use tracing::debug;

use crate::controller::Controller;

use ascot_axum::device::{DeviceData, DeviceKind};

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
pub(crate) struct Device<'a> {
    // Metadata.
    pub(crate) metadata: DeviceMetadata,
    // Data.
    pub(crate) data: DeviceData<'a>,
    // Device data controller.
    pub(crate) controller: Controller,
}

impl<'a> Device<'a> {
    pub(crate) fn is_recheable(&self) -> bool {
        self.metadata
            .addresses
            .iter()
            .any(|address| address.recheable)
    }
}

pub(crate) struct DeviceRetriever(DeviceMetadata);

impl DeviceRetriever {
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

    pub(crate) async fn retrieve<'a>(mut self) -> Option<DeviceData<'a>> {
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
        device_data
    }
}
