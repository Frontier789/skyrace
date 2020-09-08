use crate::car::CarComponent;

use glui::mecs::*;
use glui::tools::serde_tools::SerdeJsonQuick;
use glui::tools::*;
use std::time::Duration;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum View {
    Racer,
    TopDown,
    Side,
}

impl View {
    pub fn cycle(self) -> Self {
        match self {
            View::Racer => View::Side,
            View::Side => View::TopDown,
            View::TopDown => View::Racer,
        }
    }
}

pub struct CamFollowCar {
    cars: Vec<Entity>,
    active_car: usize,
    cam_entity: Entity,
    view: View,
    free_view: bool,
    max_cam_speed: f32,
}

impl System for CamFollowCar {
    fn update(&mut self, delta_time: Duration, world: &mut StaticWorld) {
        let dt = delta_time.as_secs_f32();

        if !self.free_view {
            let p = self.cam_should_be_pos(world);
            let car = world
                .component_mut::<CarComponent>(self.cars[self.active_car])
                .unwrap();

            let car_pos = car.pos3();

            let cam = &mut world
                .component_mut::<DataComponent<Camera>>(self.cam_entity)
                .unwrap()
                .data;

            let cam_p = cam.params.spatial.pos;
            let delta = p - cam_p;
            let d = delta.length();
            let step = (33.0 + d * 0.4 + d * d * 0.3).min(self.max_cam_speed) * dt;

            let new_pos = if d < step || d < 0.001 {
                p
            } else {
                let dir = delta.sgn();
                cam_p + dir * step
            };

            let t = cam.params.spatial.target;
            let delta = car_pos - t;
            let d = delta.length();
            let new_target = if d < step || d < 0.001 {
                car_pos
            } else {
                let dir = delta.sgn();
                t + dir * step
            };

            self.max_cam_speed += 10000.0f32.max(self.max_cam_speed * 3.0).min(100.0) * dt;

            cam.params
                .look_at(new_pos, new_target, Vec3::new(0.0, 1.0, 0.0));
        }
    }
    fn window_event(&mut self, event: &GlutinWindowEvent, world: &mut StaticWorld) -> bool {
        if let GlutinWindowEvent::KeyboardInput { input, .. } = event {
            let press = input.state == GlutinElementState::Pressed;
            if let Some(key) = input.virtual_keycode {
                if key == GlutinKey::T && !press {
                    let cam = &mut world
                        .component_mut::<DataComponent<Camera>>(self.cam_entity)
                        .unwrap()
                        .data;

                    if self.free_view {
                        self.free_view = false;
                        cam.set_controller(NoController {});
                    } else {
                        self.view = self.view.cycle();
                    }
                    self.max_cam_speed = 0.0;
                }
                if key == GlutinKey::Y && !press {
                    let cam = &mut world
                        .component_mut::<DataComponent<Camera>>(self.cam_entity)
                        .unwrap()
                        .data;

                    if self.free_view {
                        self.free_view = false;
                        cam.set_controller(NoController {});
                    } else {
                        self.free_view = true;
                        let mut controller = ModelViewController::new(Vec2::new(1024.0, 768.0));
                        *controller.spatial_mut() = cam.params.spatial;
                        controller.disable_roll = true;
                        cam.set_controller(controller);
                    }
                    self.max_cam_speed = 0.0;
                }
                if key == GlutinKey::LBracket && !press {
                    self.active_car = (self.active_car + 1) % self.cars.len();
                    self.max_cam_speed = 0.0;
                }
                if key == GlutinKey::RBracket && !press {
                    self.active_car = (self.active_car + self.cars.len() - 1) % self.cars.len();
                    self.max_cam_speed = 0.0;
                }
            }
        }

        false
    }
}

impl CamFollowCar {
    fn cam_should_be_pos(&self, world: &mut StaticWorld) -> Vec3 {
        let car = world
            .component_mut::<CarComponent>(self.cars[self.active_car])
            .unwrap();

        let mut p = car.pos3();
        let r = car.right3();
        match self.view {
            View::Racer => {
                p += Vec3::new(0.0, 5.0, 0.0) - car.dir3() * 10.0;
            }
            View::Side => {
                p += Vec3::new(0.0, 2.0, 0.0) + r * 7.0;
            }
            View::TopDown => {
                p += Vec3::new(0.0, 17.0, 0.0) - car.dir3() * 0.5;
            }
        }

        p
    }
    fn init_cam(&self, world: &mut StaticWorld) {
        let car = world
            .component_mut::<CarComponent>(self.cars[self.active_car])
            .unwrap();

        let car_p = car.pos3();
        let p = self.cam_should_be_pos(world) + Vec3::new(0.0, 5.0, 0.0);

        let cam = &mut world
            .component_mut::<DataComponent<Camera>>(self.cam_entity)
            .unwrap()
            .data;

        if self.free_view {
            let mut controller = ModelViewController::new(Vec2::new(1024.0, 768.0));
            controller.disable_roll = true;
            if let Ok(spatial) = CameraSpatialParams::load_json("cam.json") {
                *controller.spatial_mut() = spatial;
            }
            cam.set_controller(controller);
        } else {
            cam.params.look_at(p, car_p, Vec3::new(0.0, 1.0, 0.0));
        }
    }
    pub fn new(
        cars: Vec<Entity>,
        camera: Entity,
        free_view: bool,
        world: &mut StaticWorld,
    ) -> CamFollowCar {
        let me = CamFollowCar {
            cars,
            active_car: 0,
            cam_entity: camera,
            view: View::Racer,
            free_view,
            max_cam_speed: 0.0,
        };
        me.init_cam(world);
        me
    }
}
