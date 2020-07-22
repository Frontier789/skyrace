#![feature(clamp)]

mod camera_on_car;
mod playback_driver;
mod spline_editor;

mod car;
mod car_driver;
mod ground;
mod gui;
mod line_system;
mod sky;

extern crate glui;
extern crate glui_proc;

use crate::camera_on_car::CamFollowCar;
use crate::car::{CarComponent, CarSystem};
use crate::car_driver::CarDriver;
use crate::ground::Ground;
use crate::gui::Gui;
use crate::line_system::LineSystem;
use crate::playback_driver::{CarDriveState, PlaybackDriver};
use crate::sky::Sky;
use glui::mecs::World;
use glui::mecs::*;
use glui::tools::serde_tools::SerdeJsonQuick;
use glui::tools::*;

fn main() {
    let mut w: World = World::new_win(Vec2::new(1024.0, 768.0), "Skyracer", Vec3::grey(0.1));

    let ds = DrawSystem::new(&mut w, NoController {});
    let camera_entity = ds.camera_entity;

    w.add_system(ds);

    let carsys = CarSystem::new();
    let linesys = LineSystem::new(w.as_static_mut());
    let ground = Ground::new(w.as_static_mut());

    if let Ok(path) = Vec::<CarDriveState>::load_json("path.json") {
        let car = carsys.create_car(w.as_static_mut());
        let driver = PlaybackDriver::new(car, path);
        w.add_system(driver);
    }

    let car = carsys.create_car(w.as_static_mut());
    w.component_mut::<CarComponent>(car).unwrap().position = Vec2::new(0.0, 2.0);
    let driver = CarDriver::new(car);
    w.add_system(driver);

    let follower = CamFollowCar::new(car, camera_entity, true, w.as_static_mut());
    w.add_system(follower);
    w.add_gui(Gui::from_car(car));

    w.add_system(ground);
    w.add_system(linesys);
    w.add_system(carsys);

    let sky = Sky::new(Vec3::new(1.0, 0.3, 1.0).sgn(), &mut w);
    w.add_system(sky);

    w.run();

    let cam = &mut w
        .component_mut::<DataComponent<Camera>>(camera_entity)
        .unwrap()
        .data;

    cam.params.spatial.save_json("cam.json").unwrap();
}
