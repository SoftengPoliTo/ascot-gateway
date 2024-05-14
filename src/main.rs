#[macro_use]
extern crate rocket;

mod database;
mod error;
mod widgets;

use std::time::Duration;

// Ascot library
use ascot_library::hazards::Hazard;

// Service protocol: mDNS-SD
use mdns_sd::{Receiver, ServiceDaemon, ServiceEvent};

// Web app
use rocket::http::uri::Origin;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;

// Templates engine
use rocket_dyn_templates::{context, Template};

// Database
use rocket_db_pools::Connection;

use crate::database::{
    device::{Device, DeviceHazard},
    query::{all_hazards, clear_database, insert_address, insert_device, insert_property},
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

// Save discovered devices in database.
async fn save_devices(
    receiver: Receiver<ServiceEvent>,
    mut db: Connection<Devices>,
    uri: &Origin<'_>,
) -> Result<(), InternalError> {
    // Run for 1 second in search of devices and saves them into the database.
    while let Ok(event) = receiver.recv_timeout(Duration::from_secs(1)) {
        if let ServiceEvent::ServiceResolved(info) = event {
            // Device addresses
            let addresses = info.get_addresses();

            // Check whether there are device addresses.
            //
            // If no address has been found, return an error.
            if addresses.is_empty() {
                return Err(InternalError::text(uri, "No device address available"));
            }

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
            let id =
                query_error(insert_device(&mut db, info.get_port(), scheme, path), uri).await?;

            // Save addresses
            for address in addresses {
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
    }
    Ok(())
}

// Retrieve hazards.
async fn get_hazards(
    db: Connection<Devices>,
    uri: &Origin<'_>,
) -> Result<Vec<DeviceHazard>, InternalError> {
    let hazards = query_error(all_hazards(db), uri).await?;
    Ok(hazards
        .into_iter()
        .filter_map(|id| Hazard::from_id(id).map(|hazard| DeviceHazard::new(id, hazard.name())))
        .collect())
}

// Find devices in the network and save them into the database.
#[put("/")]
async fn devices_discovery(
    state: &State<ServiceState>,
    mut db: Connection<Devices>,
    uri: &Origin<'_>,
) -> Result<Redirect, InternalError> {
    // Browse the network in search of a service type.
    let receiver = state
        .0
        .browse(SERVICE_TYPE)
        .map_err(|e| InternalError::text(uri, &e.to_string()))?;

    // Clear the database
    query_error(clear_database(&mut db), uri).await?;

    // Run in search of devices and saves them into the database.
    save_devices(receiver, db, uri).await?;

    // Redirect to index
    Ok(Redirect::to(uri!(index)))
}

#[get("/")]
async fn index<'a>(
    mut db: Connection<Devices>,
    uri: &Origin<'_>,
) -> Result<Template, InternalError> {
    let first = true;
    let devices: Vec<Device> = if first {
        query_error(Device::search_for_devices(&mut db), uri).await?
    } else {
        Vec::new()
        //query_error(Device::read_from_database(db), uri).await?
    };

    // Retrieve all hazards contained in the database.
    let hazards = get_hazards(db, uri).await?;

    Ok(Template::render(
        "index",
        context! {
          no_devices_message: devices.is_empty().then_some("No devices available!"),
          devices,
          hazards,
          discover_route: uri!(devices_discovery),
          discover_message: "Discover devices",

        },
    ))
}

// Return the detected devices routes for possible third-parties applications.
#[get("/devices")]
async fn devices<'a>(
    db: Connection<Devices>,
    uri: &Origin<'_>,
) -> Result<Json<Vec<Device<'a>>>, InternalError> {
    //let devices = query_error(Device::obtain_routes(db), uri).await?;
    let devices: Vec<Device> = Vec::new();
    Ok(Json(devices))
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
        .mount("/", routes![index, devices, devices_discovery])
        .manage(ServiceState(mdns))
        .attach(database::stage())
        .attach(Template::fairing())
        .register("/", error::catchers())
}
