use std::collections::HashMap;

use rocket::form::FromForm;

#[derive(Debug, FromForm)]
struct Data<T> {
    #[field(name = "route")]
    route_id: u16,
    val: T,
}

#[derive(Debug, FromForm)]
pub(crate) struct DeviceData<'r> {
    #[field(name = "slidersu64")]
    pub(crate) sliders_u64: HashMap<&'r str, Data<u64>>,
    #[field(name = "slidersf64")]
    pub(crate) sliders_f64: HashMap<&'r str, Data<f64>>,
    #[field(name = "checkboxes")]
    pub(crate) checkboxes: HashMap<&'r str, Data<bool>>,
    #[field(name = "buttons")]
    pub(crate) buttons: HashMap<&'r str, Data<bool>>,
}
