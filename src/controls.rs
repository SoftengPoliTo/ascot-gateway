use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Button {
    name: &'static str,
    with_state: bool,
}

impl Button {
    pub(crate) fn with_state(name: &'static str) -> Self {
        Self {
            name,
            with_state: true,
        }
    }

    pub(crate) fn without_state(name: &'static str) -> Self {
        Self {
            name,
            with_state: false,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Slider<T> {
    name: &'static str,
    min: T,
    max: T,
    step: T,
    value: T,
}

impl<T> Slider<T> {
    pub(crate) fn new(name: &'static str, min: T, max: T, step: T, value: T) -> Self {
        Self {
            name,
            min,
            max,
            step,
            value,
        }
    }
}

// Slider type.
#[derive(Serialize)]
pub(crate) enum SliderType {
    // Integer type.
    U64(Slider<u64>),
    // Float type.
    F64(Slider<f64>),
}

#[derive(Serialize)]
pub(crate) struct CheckBox {
    name: &'static str,
    value: &'static str,
}

impl CheckBox {
    pub(crate) fn new(name: &'static str, value: &'static str) -> Self {
        Self { name, value }
    }
}

// A form control.
//
// According to the REST API, needed inputs.
#[derive(Serialize)]
pub(crate) enum Control {
    // A button.
    Button(Button),
    // A slider.
    Slider(SliderType),
    // A checkbox.
    CheckBox(CheckBox),
}

// Search for route id associated with the route, look for inputs for the route,
// send route.
