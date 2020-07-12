use crate::car::CarComponent;
use glui::gui::{gui_primitives, Align, GridLayout, GuiBuilder, GuiDimension, Text};
use glui::mecs::{Entity, StaticWorld};
use glui::tools::Vec4;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, PartialEq, Debug)]
pub struct Gui {
    pub speed: f32,
    pub car: Entity,
}

#[allow(unused_must_use)]
impl GuiBuilder for Gui {
    fn build(&self) {
        let mut disp = HashMap::new();
        disp.insert(
            "Speed: ",
            format!("{} km/h", (self.speed * 3.6 * 10.0).round() / 10.0),
        );

        -GridLayout {
            row_heights: vec![GuiDimension::Units(400.0)],
            col_widths: vec![GuiDimension::Units(200.0)],
            ..Default::default()
        } << gui_primitives::build_table_proto(
            20.0,
            &disp,
            Text {
                align: Align::left(),
                color: Vec4::WHITE,
                ..Default::default()
            },
            Text {
                align: Align::left(),
                color: Vec4::WHITE,
                ..Default::default()
            },
        );
    }

    fn update(&mut self, _delta_time: Duration, world: &mut StaticWorld) {
        let car = world.component_mut::<CarComponent>(self.car).unwrap();

        self.speed = car.velocity.length();
    }
}
