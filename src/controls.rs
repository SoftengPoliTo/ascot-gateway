use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct Button {
    name: String,
    with_state: bool,
}

impl Button {
    pub(crate) fn init(name: String) -> Self {
        Self {
            name,
            with_state: false,
        }
    }

    pub(crate) fn with_state(name: String) -> Self {
        Self {
            name,
            with_state: true,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Slider<T> {
    name: String,
    min: T,
    max: T,
    step: T,
    value: T,
}

impl<T> Slider<T> {
    pub(crate) fn new(name: String, min: T, max: T, step: T, value: T) -> Self {
        Self {
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
    name: String,
    value: bool,
}

impl CheckBox {
    pub(crate) fn init(name: String) -> Self {
        Self { name, value: false }
    }

    pub(crate) fn checked(name: String) -> Self {
        Self { name, value: true }
    }
}

// Search for route id associated with the route, look for inputs for the route,
// send route.
