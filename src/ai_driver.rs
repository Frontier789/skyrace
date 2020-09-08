use crate::car::CarComponent;

extern crate serde;
extern crate serde_json;

use crate::track::Track;
use glui::mecs::*;
use glui::tools::serde_tools::SerdeError;
use glui::tools::{LinSpace, Randable, Smoothstep, Vec2};
use std::time::{Duration, Instant};

struct AiTarget {
    pub offset: Vec2,
    pub steer_coef: f32,
    pub timer: Instant,
    pub next_update: Duration,
}

impl AiTarget {
    pub fn new() -> AiTarget {
        AiTarget {
            offset: Vec2::zero(),
            steer_coef: 2.0,
            timer: Instant::now(),
            next_update: Duration::from_millis(1500),
        }
    }
}

pub struct AiDriver {
    car: Entity,
    track_points: Vec<Vec2>,
    next_pt: usize,
    target: AiTarget,
}

impl System for AiDriver {
    fn update(&mut self, _delta_time: Duration, world: &mut StaticWorld) {
        let mut car = world.component_mut::<CarComponent>(self.car).unwrap();

        let target = self.track_points[self.next_pt] + self.target.offset;
        let v = target - car.position;
        let d = car.dir();

        let steer = v.sgn().dot(d.perp()) * self.target.steer_coef;
        car.steer = (steer - car.steer).max(-0.1).min(0.1) + car.steer;
        car.brake = 0.0;
        car.throttle = car.speed().smoothstep(46.0, 48.0) * -0.5 + 1.0;

        if v.length() < 3.0 {
            self.next_pt = (self.next_pt + 1) % self.track_points.len();
        }

        if self.target.timer.elapsed() > self.target.next_update {
            self.target.offset = Vec2::unit_rand() * 1.0 - Vec2::new(1.0, 1.0) * 0.5;
            self.target.steer_coef = f32::unit_rand() * 1.0 + 1.5;
            self.target.next_update = Duration::from_secs_f32(f32::unit_rand() * 0.1 + 0.9);
            self.target.timer = Instant::now();
        }
    }

    fn window_event(&mut self, event: &GlutinWindowEvent, world: &mut StaticWorld) -> bool {
        let car = world.component_mut::<CarComponent>(self.car).unwrap();

        if let GlutinWindowEvent::KeyboardInput { input, .. } = event {
            let press = input.state == GlutinElementState::Pressed;
            if let Some(key) = input.virtual_keycode {
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

impl AiDriver {
    pub fn new(car_entity: Entity, lane: f32) -> Result<AiDriver, SerdeError> {
        let data = Track::track_curve_control_points()?;
        let mut i = 0;
        let mut pts = vec![];
        let track_width = Track::track_width();
        while i + 4 <= data.len() {
            let p0 = data[i + 0];
            let p1 = data[i + 1];
            let p2 = data[i + 2];
            let p3 = data[i + 3];

            for t in (0.0..1.0).linspace(5) {
                let (p, _v, n) = Vec2::eval_bezier4(p0, p1, p2, p3, t);
                pts.push(p + n * track_width / 7.0 * lane);
            }

            i += 3;
        }
        Ok(AiDriver {
            car: car_entity,
            track_points: pts,
            next_pt: 0,
            target: AiTarget::new(),
        })
    }
}
