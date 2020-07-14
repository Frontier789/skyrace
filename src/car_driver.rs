use crate::car::CarComponent;

use glui::mecs::*;
use glui::tools::*;
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub enum Steering {
    Left,
    Right,
    None,
}

pub struct CarDriver {
    pub car: Entity,
    pub cam_entity: Entity,
    pub view: i32,
    pub braking: bool,
    pub throttling: bool,
    pub steering: Steering,
}

impl System for CarDriver {
    fn update(&mut self, delta_time: Duration, world: &mut StaticWorld) {
        let mut car = world.component_mut::<CarComponent>(self.car).unwrap();

        let dt = delta_time.as_secs_f32();

        match self.steering {
            Steering::Left => {
                car.steer = (car.steer - 4.0 * dt).clamp(-1.0, 1.0);
            }
            Steering::Right => {
                car.steer = (car.steer + 4.0 * dt).clamp(-1.0, 1.0);
            }
            Steering::None => {
                if car.steer < 0.0 {
                    car.steer = (car.steer + 3.0 * dt).min(0.0);
                }
                if car.steer > 0.0 {
                    car.steer = (car.steer - 3.0 * dt).max(0.0);
                }
            }
        }

        car.brake = if self.braking { 1.0 } else { 0.0 };
        car.throttle = if self.throttling { 1.0 } else { 0.0 };

        let d = car.velocity.length() / 55.0;
        let r = car.right3();

        let mut p = Vec3::new(car.position.x, 0.0, car.position.y);
        match self.view {
            0 => {
                p += Vec3::new(0.0, 5.0, 0.0) - car.dir3() * (10.0 + d * 0.0) + r * 3.0;
            }
            1 => {
                p += Vec3::new(0.0, 2.0, 0.0) + r * 7.0;
            }
            2 => {
                p += Vec3::new(0.0, 17.0, 0.0) - car.dir3() * 0.1;
            }
            _ => {}
        }

        let car_pos = car.pos3();

        let cam = &mut world
            .component_mut::<DataComponent<Camera>>(self.cam_entity)
            .unwrap()
            .data;

        if let Some(_) = cam.controller::<NoController>() {
            cam.params.look_at(p, car_pos, Vec3::new(0.0, 1.0, 0.0));
        }
    }
    fn window_event(&mut self, event: &GlutinWindowEvent, world: &mut StaticWorld) -> bool {
        let car = world.component_mut::<CarComponent>(self.car).unwrap();

        if let GlutinWindowEvent::KeyboardInput { input, .. } = event {
            let press = input.state == GlutinElementState::Pressed;
            if let Some(key) = input.virtual_keycode {
                if key == GlutinKey::W {
                    self.throttling = press;
                }
                if key == GlutinKey::S {
                    self.braking = press;
                }
                match (self.steering, key, press) {
                    (Steering::None, GlutinKey::A, true) => self.steering = Steering::Left,
                    (Steering::None, GlutinKey::D, true) => self.steering = Steering::Right,
                    (Steering::Right, GlutinKey::A, true) => self.steering = Steering::None,
                    (Steering::Left, GlutinKey::D, true) => self.steering = Steering::None,
                    (Steering::Left, GlutinKey::A, false) => self.steering = Steering::None,
                    (Steering::Right, GlutinKey::D, false) => self.steering = Steering::None,
                    (Steering::None, GlutinKey::A, false) => self.steering = Steering::Right,
                    (Steering::None, GlutinKey::D, false) => self.steering = Steering::Left,
                    _ => {}
                }
                if key == GlutinKey::T && !press {
                    self.view = (self.view + 1) % 3;
                }
                if key == GlutinKey::R && !press {
                    car.position = Vec2::origin();
                    car.velocity = Vec2::zero();
                    car.acceleration = Vec2::zero();
                    car.heading = 0.0;
                    car.angular_velocity = 0.0;
                }
            }
        }

        false
    }
}

impl CarDriver {
    pub fn new(car: Entity, camera: Entity) -> CarDriver {
        CarDriver {
            car,
            cam_entity: camera,
            view: 0,
            braking: false,
            throttling: false,
            steering: Steering::None,
        }
    }
}
