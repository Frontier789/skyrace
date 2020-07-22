use crate::car::CarComponent;

extern crate serde;
extern crate serde_json;

use glui::mecs::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct CarDriveState {
    pub brake: f32,
    pub throttle: f32,
    pub steer: f32,
    pub update_id: usize,
}

impl PartialEq for CarDriveState {
    fn eq(&self, other: &Self) -> bool {
        self.brake.eq(&other.brake)
            && self.throttle.eq(&other.throttle)
            && self.steer.eq(&other.steer)
    }
}

pub struct PlaybackDriver {
    car: Entity,
    state_stack: Vec<CarDriveState>,
    update_id: usize,
}

impl System for PlaybackDriver {
    fn update(&mut self, _delta_time: Duration, world: &mut StaticWorld) {
        let mut car = world.component_mut::<CarComponent>(self.car).unwrap();

        if let Some(state) = self.state_stack.last().cloned() {
            if state.update_id == self.update_id {
                car.steer = state.steer;
                car.brake = state.brake;
                car.throttle = state.throttle;
                self.state_stack.pop();
                self.update_id = 0;
            } else {
                self.update_id += 1;
            }
        }
    }
}

impl PlaybackDriver {
    pub fn new(car: Entity, mut state_queue: Vec<CarDriveState>) -> PlaybackDriver {
        state_queue.reverse();

        PlaybackDriver {
            car,
            state_stack: state_queue,
            update_id: 0,
        }
    }
}
