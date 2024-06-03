use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct Button {
    route_id: u16,
    name: String,
    with_state: bool,
}

impl Button {
    pub(crate) fn init(route_id: u16, name: String) -> Self {
        Self {
            route_id,
            name,
            with_state: false,
        }
    }

    pub(crate) fn with_state(route_id: u16, name: String) -> Self {
        Self {
            route_id,
            name,
            with_state: true,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Slider<T> {
    route_id: u16,
    name: String,
    min: T,
    max: T,
    step: T,
    value: T,
}

impl<T> Slider<T> {
    pub(crate) fn new(route_id: u16, name: String, min: T, max: T, step: T, value: T) -> Self {
        Self {
            route_id,
            name,
            min,
            max,
            step,
            value,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct CheckBox {
    route_id: u16,
    name: String,
    value: bool,
}

impl CheckBox {
    pub(crate) fn init(route_id: u16, name: String) -> Self {
        Self {
            route_id,
            name,
            value: false,
        }
    }

    pub(crate) fn checked(route_id: u16, name: String) -> Self {
        Self {
            route_id,
            name,
            value: true,
        }
    }
}
