use crate::car::CarComponent;

use crate::playback_driver::CarDriveState;
use glui::mecs::*;
use glui::tools::serde_tools::SerdeJsonQuick;
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
    pub braking: bool,
    pub throttling: bool,
    pub steering: Steering,
    pub state: CarDriveState,
    pub update_id: usize,
    pub states: Vec<CarDriveState>,
}

impl System for CarDriver {
    fn update(&mut self, delta_time: Duration, world: &mut StaticWorld) {
        let mut car = world.component_mut::<CarComponent>(self.car).unwrap();

        let dt = delta_time.as_secs_f32();

        match self.steering {
            Steering::Left => {
                car.steer = (car.steer - 2.0 * dt).clamp(-1.0, 1.0);
            }
            Steering::Right => {
                car.steer = (car.steer + 2.0 * dt).clamp(-1.0, 1.0);
            }
            Steering::None => {
                if car.steer < 0.0 {
                    car.steer = (car.steer + 2.0 * dt).min(0.0);
                }
                if car.steer > 0.0 {
                    car.steer = (car.steer - 2.0 * dt).max(0.0);
                }
            }
        }

        car.brake = if self.braking { 1.0 } else { 0.0 };
        car.throttle = if self.throttling { 1.0 } else { 0.0 };

        let new_state = CarDriveState {
            throttle: car.throttle,
            steer: car.steer,
            brake: car.brake,
            update_id: self.update_id,
        };

        if self.state != new_state {
            self.states.push(self.state);
            self.state = new_state;
            self.update_id = 0;
        } else {
            self.update_id += 1;
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
                if key == GlutinKey::R && !press {
                    car.position = Vec2::origin();
                    car.velocity = Vec2::zero();
                    car.acceleration = Vec2::zero();
                    car.heading = 0.0;
                    car.angular_velocity = 0.0;
                    self.states = vec![];
                    self.update_id = 0;
                }
                if key == GlutinKey::P && !press {
                    self.states.save_json("path.json").unwrap();
                }
            }
        }

        false
    }
}

impl CarDriver {
    pub fn new(car: Entity) -> CarDriver {
        CarDriver {
            car,
            braking: false,
            throttling: false,
            steering: Steering::None,
            state: Default::default(),
            update_id: 0,
            states: vec![],
        }
    }
}

impl Drop for CarDriver {
    fn drop(&mut self) {
        self.states.save_json("last_path.json").unwrap();
    }
}
