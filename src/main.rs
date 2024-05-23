#[macro_use]
extern crate rocket;

mod controls;
mod database;
mod error;

use std::time::Duration;

// Service protocol: mDNS-SD
use mdns_sd::{Receiver, ServiceDaemon, ServiceEvent, ServiceInfo};

// Web app
use rocket::http::uri::Origin;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;

// Templates engine
use rocket_dyn_templates::{context, Template};

// Database
use rocket_db_pools::Connection;

// Tracing
use tracing::warn;

use crate::database::{
    device::Device,
    query::{clear_database, insert_address, insert_device, insert_property},
    Devices,
};
use crate::error::{query_error, InternalError};

// Ascot service type.
const SERVICE_TYPE: &str = "_ascot._tcp.local.";

// Default scheme is `http`.
const DEFAULT_SCHEME: &str = "http";

// Well-known URI.
// https://en.wikipedia.org/wiki/Well-known_URI
//
// Requests to the servers for well-known services or information are available
// at URLs consistent well-known locations across servers.
const WELL_KNOWN_URI: &str = "/.well-known/ascot";

// Cookies key
const DB: &str = "db";

// Search ascot devices.
async fn search_devices(receiver: Receiver<ServiceEvent>) -> Vec<ServiceInfo> {
    let mut devices_info = Vec::new();
    // Run for 1 second in search of devices and returns their information.
    while let Ok(event) = receiver.recv_timeout(Duration::from_secs(1)) {
        if let ServiceEvent::ServiceResolved(info) = event {
            // Check whether there are device addresses.
            //
            // If no address has been found, prints a warning and continue the
            // loop.
            if info.get_addresses().is_empty() {
                // TODO: We should implement a logger to show this messages
                // directly in the gateway.
                warn!("No device address available for {:?}", info);
                continue;
            }

            // Save discovered devices information.
            devices_info.push(info);
        }
    }
    devices_info
}

// Save discovered devices into the database.
async fn save_devices(
    mut db: Connection<Devices>,
    devices_info: Vec<ServiceInfo>,
    uri: &Origin<'_>,
) -> Result<(), InternalError> {
    for info in devices_info {
        // Device properties.
        let properties = info.get_properties();

        // Internet scheme.
        //
        // If no scheme has been found, use `http` as default scheme.
        let scheme = properties
            .get_property_val_str("scheme")
            .unwrap_or(DEFAULT_SCHEME);

        // Resource path.
        //
        // If no path has been found, use the well-known URI as default
        // path.
        let path = properties
            .get_property_val_str("path")
            .unwrap_or(WELL_KNOWN_URI);

        // Insert device into the database and get back its identifier
        let id = query_error(insert_device(&mut db, info.get_port(), scheme, path), uri).await?;

        // Save addresses
        for address in info.get_addresses() {
            query_error(insert_address(&mut db, address.to_string(), id), uri).await?;
        }

        // Save properties
        for property in properties.iter() {
            query_error(
                insert_property(&mut db, property.key(), property.val_str(), id),
                uri,
            )
            .await?;
        }
    }
    Ok(())
}

// Find devices in the network and
// save their metadata into the database.
#[put("/")]
async fn devices_discovery(
    state: &State<ServiceState>,
    mut db: Connection<Devices>,
    jar: &CookieJar<'_>,
    uri: &Origin<'_>,
) -> Result<Redirect, InternalError> {
    // Browse the network in search of the input service type.
    let receiver = state
        .0
        .browse(SERVICE_TYPE)
        .map_err(|e| InternalError::text(uri, &e.to_string()))?;

    // If a service type has been found, search devices and their metadata.
    let devices_info = search_devices(receiver).await;

    // If some devices have been found, delete every old device from the
    // database and insert every discovered devices.
    if !devices_info.is_empty() {
        // Clear the database
        query_error(clear_database(&mut db), uri).await?;

        // Save devices into the database.
        save_devices(db, devices_info, uri).await?;

        // The discovery phase is completed and every device has been
        // registered into the database.
        //
        // Sets the cookie value to state that the database has been reset.
        jar.remove_private(DB);
    }

    // Redirect to index
    Ok(Redirect::to(uri!(index)))
}

#[get("/")]
async fn index<'a>(
    mut db: Connection<Devices>,
    jar: &CookieJar<'_>,
    uri: &Origin<'_>,
) -> Result<Template, InternalError> {
    let is_db_init = jar.get_private(DB).is_none();

    // Contact discovered devices with the goal of retrieving their data and
    // building their controls.
    let devices = if is_db_init {
        //query_error(Device::search_for_devices(&mut db), uri).await?
        let devices = vec![Device::fake_device1(), Device::fake_device2()];

        // Sets the cookie value to state that the database
        // has been initialized.
        jar.add_private((DB, "1"));

        devices
    } else {
        //query_error(Device::read_from_database(db), uri).await?
        vec![Device::fake_device1(), Device::fake_device2()]
    };

    Ok(Template::render(
        "index",
        context! {
          no_devices_message: devices.is_empty().then_some("No devices available!"),
          devices,
          discover_route: uri!(devices_discovery),
          discover_message: "Discover devices",

        },
    ))
}

use rocket::form::{self, DataField, Error, Errors, Form, FromForm, ValueField};

// Inspects changed device data.
//
// 1. Build a REST request to a device with the data passed as input.
// 2. Send the request to a device with the modified data.
// 3. Save new data into the database.
// 4. Go to the index
#[put("/device/<id>", data = "<input>")]
async fn device_request<'v>(
    id: u16,
    input: Form<Vec<u16>>,
    db: Connection<Devices>,
    uri: &Origin<'_>,
) -> Result<Redirect, InternalError> {
    // Build a REST request from data passed as input.

    // Send the request

    // Save into the database the new data

    // Redirect to index
    Ok(Redirect::to(uri!(index)))
}

// Service state.
struct ServiceState(ServiceDaemon);

#[launch]
fn rocket() -> _ {
    // Enable tracing subscriber
    tracing_subscriber::fmt().init();

    // Create a daemon
    let mdns = ServiceDaemon::new().expect("Failed to create mdns daemon");

    rocket::build()
        .mount("/", routes![index, devices_discovery, device_request])
        .manage(ServiceState(mdns))
        .attach(database::stage())
        .attach(Template::fairing())
        .register("/", error::catchers())
}
