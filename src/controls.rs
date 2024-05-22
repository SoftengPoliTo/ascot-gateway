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
    value: &'static str,
}

impl CheckBox {
    pub(crate) fn new(name: String, value: &'static str) -> Self {
        Self { name, value }
    }
}

// Search for route id associated with the route, look for inputs for the route,
// send route.
