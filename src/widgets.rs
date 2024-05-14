use serde::Serialize;

// Light device controller.
#[derive(Debug, Serialize)]
pub(crate) struct Light {
    pub(crate) on: bool,
    pub(crate) toggle: bool,
}

// Fridge device controller.
#[derive(Debug, Serialize)]
pub(crate) struct Fridge {
    pub(crate) camera: bool,
}

// A device controller.
//
// It determines how device data should be presented graphically.
#[derive(Debug, Serialize)]
pub(crate) enum Controller {
    // Light controller.
    Light(Light),
    // Fridge controller.
    Fridge(Fridge),
}
// TODO: Considering device state to change controller state
impl Controller {
    pub(crate) fn light() -> Self {
        Self::Light(Light {
            on: false,
            toggle: false,
        })
    }

    pub(crate) fn fridge() -> Self {
        Self::Fridge(Fridge { camera: false })
    }
}

// Search for route id associated with the route, look for inputs for the route,
// send route.
