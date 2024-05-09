use rocket_db_pools::{sqlx, Connection};

use super::{Address, Devices, Info, RangeInputF64, RangeInputU64};

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
    hazard: &str,
    device_id: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO hazards(hazard, device_id) VALUES ($1, $2)")
        .bind(hazard)
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

// Return device information.
#[inline(always)]
pub(crate) async fn select_device_info(
    db: &mut Connection<Devices>,
) -> Result<Vec<Info>, sqlx::Error> {
    sqlx::query_as("SELECT id, port, scheme, path FROM devices ORDER BY id")
        .fetch_all(&mut ***db)
        .await
}

// Return device address information.
#[inline(always)]
pub(crate) async fn select_device_addresses(
    db: &mut Connection<Devices>,
    device_id: u16,
) -> Result<Vec<Address>, sqlx::Error> {
    sqlx::query_as("SELECT address FROM addresses WHERE device_id = $1")
        .bind(device_id)
        .fetch_all(&mut ***db)
        .await
}
