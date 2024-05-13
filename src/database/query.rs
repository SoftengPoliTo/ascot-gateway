use rocket_db_pools::{sqlx, sqlx::FromRow, Connection};

use super::{Address, Devices, Metadata, RangeInputF64, RangeInputU64};

// Insert a device in the database returning the associated identifier.
pub(crate) async fn insert_device(
    db: &mut Connection<Devices>,
    port: u16,
    scheme: &str,
    path: &str,
) -> Result<u16, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO devices(port, scheme, path) VALUES ($1, $2, $3) RETURNING id")
        .bind(port)
        .bind(scheme)
        .bind(path)
        .fetch_one(&mut ***db)
        .await
}

// Insert device address.
pub(crate) async fn insert_address(
    db: &mut Connection<Devices>,
    address: String,
    device_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO addresses(address, device_id) VALUES ($1, $2)")
        .bind(address)
        .bind(device_id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert device properties.
pub(crate) async fn insert_property(
    db: &mut Connection<Devices>,
    key: &str,
    value: &str,
    device_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO properties(key, value, device_id) VALUES ($1, $2, $3)")
        .bind(key)
        .bind(value)
        .bind(device_id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert device hazard.
pub(crate) async fn insert_hazard(
    db: &mut Connection<Devices>,
    hazard_id: u16,
    device_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO hazards(hazard_id, device_id) VALUES ($1, $2)")
        .bind(hazard_id)
        .bind(device_id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert device route.
pub(crate) async fn insert_route(
    db: &mut Connection<Devices>,
    route: &str,
    device_id: u16,
) -> Result<u16, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO routes(route, device_id) VALUES ($1, $2) RETURNING id")
        .bind(route)
        .bind(device_id)
        .fetch_one(&mut ***db)
        .await
}

// Insert boolean input for a device.
pub(crate) async fn insert_boolean_input(
    db: &mut Connection<Devices>,
    name: &str,
    default: bool,
    value: bool,
    route_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO booleans(name, default, value, route_id) VALUES ($1, $2, $3, $4)")
        .bind(name)
        .bind(default)
        .bind(value)
        .bind(route_id)
        .execute(&mut ***db)
        .await?;
    Ok(())
}

// Insert range input for u64.
pub(crate) async fn insert_rangeu64_input(
    db: &mut Connection<Devices>,
    range: RangeInputU64,
    route_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO rangesu64(name, min, max, step, default, value, route_id) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(range.name)
    .bind(range.min as i64)
    .bind(range.max as i64)
    .bind(range.step as i64)
    .bind(range.default as i64)
    .bind(range.value as i64)
    .bind(route_id)
    .execute(&mut ***db)
    .await?;
    Ok(())
}

// Insert range input for f64.
pub(crate) async fn insert_rangef64_input(
    db: &mut Connection<Devices>,
    range: RangeInputF64,
    route_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO rangesf64(name, min, max, step, default, value, route_id) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(range.name)
    .bind(range.min)
    .bind(range.max)
    .bind(range.step)
    .bind(range.default)
    .bind(range.value)
    .bind(route_id)
    .execute(&mut ***db)
    .await?;
    Ok(())
}

// Delete all data present in a database.
pub(crate) async fn clear_database(db: &mut Connection<Devices>) -> Result<(), sqlx::Error> {
    // Delete all booleans.
    sqlx::query("DELETE FROM booleans")
        .execute(&mut ***db)
        .await?;

    // Delete all u64 ranges.
    sqlx::query("DELETE FROM rangesu64")
        .execute(&mut ***db)
        .await?;

    // Delete all f64 ranges.
    sqlx::query("DELETE FROM rangesf64")
        .execute(&mut ***db)
        .await?;

    // Delete all routes.
    sqlx::query("DELETE FROM routes")
        .execute(&mut ***db)
        .await?;

    // Delete all properties.
    sqlx::query("DELETE FROM properties")
        .execute(&mut ***db)
        .await?;

    // Delete all addresses.
    sqlx::query("DELETE FROM addresses")
        .execute(&mut ***db)
        .await?;

    // Delete all hazards.
    sqlx::query("DELETE FROM hazards")
        .execute(&mut ***db)
        .await?;

    // Delete all devices.
    sqlx::query("DELETE FROM devices")
        .execute(&mut ***db)
        .await?;

    Ok(())
}

// Delete a device and its data.
pub(crate) async fn delete_device(
    db: &mut Connection<Devices>,
    id: u16,
) -> Result<(), sqlx::Error> {
    #[derive(FromRow)]
    struct RouteId(u16);

    // Get routes id associated with the device.
    let routes_id: Vec<RouteId> = sqlx::query_as("SELECT id FROM routes WHERE device_id = $1")
        .bind(id)
        .fetch_all(&mut ***db)
        .await?;

    // Delete inputs
    for route_id in routes_id {
        // Delete device booleans.
        sqlx::query("DELETE FROM booleans where route_id = $1")
            .bind(route_id.0)
            .execute(&mut ***db)
            .await?;

        // Delete device u64 ranges.
        sqlx::query("DELETE FROM rangesu64 where route_id = $1")
            .bind(route_id.0)
            .execute(&mut ***db)
            .await?;

        // Delete device f64 ranges.
        sqlx::query("DELETE FROM rangesf64 where route_id = $1")
            .bind(route_id.0)
            .execute(&mut ***db)
            .await?;
    }

    // Delete device routes.
    sqlx::query("DELETE FROM routes WHERE device_id = $1")
        .bind(id)
        .execute(&mut ***db)
        .await?;

    // Delete device properties.
    sqlx::query("DELETE FROM properties WHERE device_id = $1")
        .bind(id)
        .execute(&mut ***db)
        .await?;

    // Delete device addresses.
    sqlx::query("DELETE FROM addresses WHERE device_id = $1")
        .bind(id)
        .execute(&mut ***db)
        .await?;

    // Delete device hazards.
    sqlx::query("DELETE FROM hazards WHERE device_id = $1")
        .bind(id)
        .execute(&mut ***db)
        .await?;

    // Delete device.
    sqlx::query("DELETE FROM devices WHERE id = $1")
        .bind(id)
        .execute(&mut ***db)
        .await?;

    Ok(())
}

// Return device information.
#[inline]
pub(crate) async fn select_device_metadata(
    db: &mut Connection<Devices>,
) -> Result<Vec<Metadata>, sqlx::Error> {
    sqlx::query_as("SELECT id, port, scheme, path FROM devices ORDER BY id")
        .fetch_all(&mut ***db)
        .await
}

// Return device address information.
#[inline]
pub(crate) async fn select_device_addresses(
    db: &mut Connection<Devices>,
    device_id: u16,
) -> Result<Vec<Address>, sqlx::Error> {
    sqlx::query_as("SELECT address FROM addresses WHERE device_id = $1")
        .bind(device_id)
        .fetch_all(&mut ***db)
        .await
}

// Return all available hazards.
#[inline]
pub(crate) async fn all_hazards(mut db: Connection<Devices>) -> Result<Vec<u16>, sqlx::Error> {
    #[derive(FromRow)]
    struct HazardId(u16);

    let hazards_id: Vec<HazardId> = sqlx::query_as("SELECT DISTINCT hazard_id FROM hazards")
        .fetch_all(&mut **db)
        .await?;

    Ok(hazards_id.into_iter().map(|hazard| hazard.0).collect())
}
