use ascot_library::input::Range;

use rocket_db_pools::Connection;

use serde::Serialize;

use crate::form::{Button, CheckBox, Slider};

use super::query::{insert_boolean_input, insert_rangef64_input, insert_rangeu64_input};
use super::{Devices, RangeInputF64, RangeInputU64};

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
    pub(crate) async fn init_button(
        &mut self,
        db: &mut Connection<Devices>,
        route_name: &str,
        cleaned_route_name: String,
        route_id: u16,
    ) -> Result<(), sqlx::Error> {
        insert_boolean_input(db, route_name, false, false, route_id).await?;

        self.buttons
            .push(Button::init(route_id, cleaned_route_name));
        Ok(())
    }

    #[inline]
    pub(crate) async fn init_checkbox(
        &mut self,
        db: &mut Connection<Devices>,
        default: bool,
        route_id: u16,
        input_name: String,
    ) -> Result<(), sqlx::Error> {
        insert_boolean_input(db, &input_name, default, default, route_id).await?;
        self.checkboxes.push(CheckBox::init(route_id, input_name));
        Ok(())
    }

    #[inline]
    pub(crate) async fn init_slider_u64(
        &mut self,
        db: &mut Connection<Devices>,
        route_id: u16,
        input_name: String,
        range: &Range<u64>,
    ) -> Result<(), sqlx::Error> {
        let range_db = RangeInputU64 {
            name: input_name.clone(),
            min: range.minimum,
            max: range.maximum,
            step: range.step,
            default: range.default,
            value: range.default,
        };
        insert_rangeu64_input(db, range_db, route_id).await?;

        self.sliders_u64.push(Slider::<u64>::new(
            route_id,
            input_name,
            range.minimum,
            range.maximum,
            range.step,
            range.default,
        ));
        Ok(())
    }

    #[inline]
    pub(crate) async fn init_slider_f64(
        &mut self,
        db: &mut Connection<Devices>,
        route_id: u16,
        input_name: String,
        range: &Range<f64>,
    ) -> Result<(), sqlx::Error> {
        let range_db = RangeInputF64 {
            name: input_name.clone(),
            min: range.minimum,
            max: range.maximum,
            step: range.step,
            default: range.default,
            value: range.default,
        };
        insert_rangef64_input(db, range_db, route_id).await?;

        self.sliders_f64.push(Slider::<f64>::new(
            route_id,
            input_name,
            range.minimum,
            range.maximum,
            range.step,
            range.default,
        ));
        Ok(())
    }
}
