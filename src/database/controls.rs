use ascot_library::input::Range;

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
#[derive(Debug, Serialize, Default)]
pub(crate) struct StateControls {
    // Sliders u64.
    sliders_u64: Vec<Slider<u64>>,
    // Sliders f64.
    sliders_f64: Vec<Slider<f64>>,
    // Checkboxes.
    checkboxes: Vec<CheckBox>,
    // Buttons.
    buttons: Vec<Button>,
}

impl StateControls {
    #[inline]
    pub(crate) fn init_button(&mut self, route_id: u16, route_name: String) {
        self.buttons.push(Button::init(route_id, route_name));
    }

    #[inline]
    pub(crate) fn init_checkbox(&mut self, route_id: u16, input_name: String) {
        self.checkboxes.push(CheckBox::init(route_id, input_name));
    }

    #[inline]
    pub(crate) fn init_sliders_u64(
        &mut self,
        route_id: u16,
        input_name: String,
        range: &Range<u64>,
    ) {
        self.sliders_u64.push(Slider::<u64>::new(
            route_id,
            input_name,
            range.minimum,
            range.maximum,
            range.step,
            range.default,
        ));
    }

    #[inline]
    pub(crate) fn init_sliders_f64(
        &mut self,
        route_id: u16,
        input_name: String,
        range: &Range<f64>,
    ) {
        self.sliders_f64.push(Slider::<f64>::new(
            route_id,
            input_name,
            range.minimum,
            range.maximum,
            range.step,
            range.default,
        ));
    }
}
