use serde::Serialize;

#[derive(Debug, Serialize)]
struct Button {
    route_id: u16,
    name: String,
    with_state: bool,
}

impl Button {
    fn init(route_id: u16, name: String) -> Self {
        Self {
            route_id,
            name,
            with_state: false,
        }
    }

    fn with_state(route_id: u16, name: String) -> Self {
        Self {
            route_id,
            name,
            with_state: true,
        }
    }
}

#[derive(Debug, Serialize)]
struct Slider<T> {
    route_id: u16,
    name: String,
    min: T,
    max: T,
    step: T,
    value: T,
}

impl<T> Slider<T> {
    fn new(route_id: u16, name: String, min: T, max: T, step: T, value: T) -> Self {
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
struct CheckBox {
    route_id: u16,
    name: String,
    value: bool,
}

impl CheckBox {
    fn init(route_id: u16, name: String) -> Self {
        Self {
            route_id,
            name,
            value: false,
        }
    }

    fn checked(route_id: u16, name: String) -> Self {
        Self {
            route_id,
            name,
            value: true,
        }
    }
}
