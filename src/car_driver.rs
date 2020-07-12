use crate::car::CarComponent;

use glui::mecs::*;
use glui::tools::*;
use std::time::Duration;

pub struct CarDriver {
    pub car: Entity,
    pub cam_entity: Entity,
    pub side_view: bool,
}

impl System for CarDriver {
    fn update(&mut self, _delta_time: Duration, world: &mut StaticWorld) {
        let car = *world.component::<CarComponent>(self.car).unwrap();

        let cam = &mut world
            .component_mut::<DataComponent<Camera>>(self.cam_entity)
            .unwrap()
            .data;

        let d = car.velocity.length() / 55.0;
        let r = car.dir.cross(Vec3::new(0.0, 1.0, 0.0)).sgn();

        let mut p = car.pos;
        if !self.side_view {
            p += Vec3::new(0.0, 5.0, 0.0) - car.dir * (10.0 + d * 0.0) + r * 3.0;
        } else {
            p += Vec3::new(0.0, 2.0, 0.0) + r * 7.0;
        }

        cam.params.look_at(p, car.pos, Vec3::new(0.0, 1.0, 0.0));
    }
    fn window_event(&mut self, event: &GlutinWindowEvent, world: &mut StaticWorld) -> bool {
        let car = world.component_mut::<CarComponent>(self.car).unwrap();

        if let GlutinWindowEvent::KeyboardInput { input, .. } = event {
            let press = input.state == GlutinElementState::Pressed;
            if let Some(key) = input.virtual_keycode {
                if key == GlutinKey::W {
                    car.throttle += if press { 1.0 } else { -1.0 };
                }
                if key == GlutinKey::S {
                    car.break_ += if press { 1.0 } else { -1.0 };
                }
                if key == GlutinKey::A && press || key == GlutinKey::D && !press {
                    car.steer += 1.0;
                }
                if key == GlutinKey::A && !press || key == GlutinKey::D && press {
                    car.steer -= 1.0;
                }
                if key == GlutinKey::T && !press {
                    self.side_view = !self.side_view;
                }
            }
        }

        false
    }
}
