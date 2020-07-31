use crate::car::CarComponent;
use crate::spline_editor::SplineEditor;
use glui::gui::elements::SkipCell;
use glui::gui::{gui_primitives, Align, FontSize, GridLayout, GuiBuilder, GuiDimension, Text};
use glui::mecs::{Entity, StaticWorld};
use glui::tools::serde_tools::SerdeJsonQuick;
use glui::tools::{Vec3, Vec4};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, PartialEq, Debug)]
pub struct Gui {
    pub speed: f32,
    pub throttle: f32,
    pub steer: f32,
    pub car: Entity,
    pub track: Vec<Vec3>,
}

#[allow(unused_must_use)]
impl GuiBuilder for Gui {
    fn build(&self) {
        let mut disp = HashMap::new();
        disp.insert(
            "Speed: ",
            format!("{} km/h", (self.speed * 3.6 * 10.0).round() / 10.0),
        );
        disp.insert(
            "Throttle: ",
            format!("{}%", (self.throttle * 100.0).round()),
        );
        disp.insert(
            "Steer: ",
            format!("{}°", (self.steer * 10.0).round() / 10.0),
        );

        -GridLayout {
            row_heights: vec![
                GuiDimension::Units(400.0),
                GuiDimension::Default,
                GuiDimension::Units(200.0),
            ],
            col_widths: vec![
                GuiDimension::Units(150.0),
                GuiDimension::Default,
                GuiDimension::Units(200.0),
            ],
            ..Default::default()
        } << {
            gui_primitives::build_table_proto(
                20.0,
                &disp,
                Text {
                    align: Align::left(),
                    color: Vec4::WHITE,
                    font_size: FontSize::Em(0.75),
                    ..Default::default()
                },
                Text {
                    align: Align::left(),
                    color: Vec4::WHITE,
                    font_size: FontSize::Em(0.75),
                    ..Default::default()
                },
            );
            for _ in 0..7 {
                -SkipCell {};
            }
            -SplineEditor {
                color: Vec4::WHITE,
                points: self.track.clone(),
                callback: self
                    .make_callback2(|data, s: &SplineEditor| data.track = s.points.clone()),
                ..Default::default()
            };
        };
    }

    fn update(&mut self, _delta_time: Duration, world: &mut StaticWorld) {
        let car = world.component_mut::<CarComponent>(self.car).unwrap();

        self.speed = car.speed();
        self.throttle = car.throttle;
        self.steer = car.steer;
    }

    fn persist(&self, _world: &mut StaticWorld) {
        self.track.save_json("track.json").unwrap();
    }
    fn restore(&mut self, _world: &mut StaticWorld) {
        match Vec::<Vec3>::load_json("track.json") {
            Ok(track) => self.track = track,
            Err(_) => eprintln!("Couldn't load track.json"),
        }
    }
}

impl Gui {
    pub fn from_car(car: Entity) -> Gui {
        Gui {
            speed: 0.0,
            throttle: 0.0,
            steer: 0.0,
            car,
            track: (0..7)
                .into_iter()
                .map(|i| {
                    Vec3::new(
                        i as f32 / 6.0 * 150.0 + 25.0,
                        0.0,
                        i as f32 / 6.0 * 150.0 + 25.0,
                    )
                })
                .collect(),
        }
    }
}
