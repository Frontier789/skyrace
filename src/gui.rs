use crate::car::CarComponent;
use crate::track::{ShowWireframe, Track};
use glui::gui::{
    gui_primitives, Align, Button, ButtonBckg, FontSize, GridLayout, GuiBuilder, GuiDimension,
    Overlay, SkipCell, Text,
};
use glui::mecs::{Entity, StaticWorld};
use glui::tools::Vec4;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, PartialEq, Debug)]
pub struct Gui {
    pub speed: f32,
    pub throttle: f32,
    pub steer: f32,
    pub car: Entity,
    pub wire_frame: bool,
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
            row_heights: vec![GuiDimension::Default, GuiDimension::Units(16.0)],
            ..Default::default()
        } << {
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
                // for _ in 0..7 {
                //     -SkipCell {};
                // }
            };
            -Overlay::from(Vec4::grey(0.4)) << {
                -GridLayout {
                    col_widths: vec![GuiDimension::Default, GuiDimension::Units(20.0)],
                    ..Default::default()
                } << {
                    -SkipCell {};
                    -Button {
                        background: ButtonBckg::Fill(if self.wire_frame {
                            Vec4::WHITE
                        } else {
                            Vec4::grey(0.6)
                        }),
                        callback: self.make_callback3(|data, _btn: &Button, world| {
                            data.wire_frame = !data.wire_frame;
                            world.send_by_type::<Track, _>(ShowWireframe(data.wire_frame));
                        }),
                        ..Default::default()
                    };
                };
            };
        }
    }

    fn update(&mut self, _delta_time: Duration, world: &mut StaticWorld) {
        let car = world.component_mut::<CarComponent>(self.car).unwrap();

        self.speed = car.speed();
        self.throttle = car.throttle;
        self.steer = car.wheel_turn();
    }
}

impl Gui {
    pub fn from_car(car: Entity) -> Gui {
        Gui {
            speed: 0.0,
            throttle: 0.0,
            steer: 0.0,
            car,
            wire_frame: false,
        }
    }
}
