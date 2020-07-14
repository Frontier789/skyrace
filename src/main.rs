#![feature(clamp)]

mod car;
mod car_driver;
mod ground;
mod gui;
mod line_system;
mod sky;

extern crate glui;
extern crate glui_proc;

use crate::car::CarSystem;
use crate::car_driver::CarDriver;
use crate::ground::Ground;
use crate::gui::Gui;
use crate::line_system::LineSystem;
use crate::sky::Sky;
use glui::mecs::World;
use glui::mecs::*;
use glui::tools::*;

fn main() {
    let mut w: World = World::new_win(Vec2::new(1024.0, 768.0), "Skyracer", Vec3::grey(0.1));

    // let mut controller = ModelViewController::new(Vec2::new(640.0, 480.0));
    // controller.disable_roll = true;
    let controller = NoController {};
    let ds = DrawSystem::new(&mut w, controller);
    let camera_entity = ds.camera_entity;

    w.add_system(ds);

    let carsys = CarSystem::new();
    let car = carsys.create_car(w.as_static_mut());
    w.add_system(carsys);

    let linesys = LineSystem::new(w.as_static_mut());
    w.add_system(linesys);

    let ground = Ground::new(w.as_static_mut());
    w.add_system(ground);

    w.add_gui(Gui { speed: 0.0, car });

    let driver = CarDriver::new(car, camera_entity);
    w.add_system(driver);

    let sky = Sky::new(Vec3::new(1.0, 0.3, 1.0).sgn(), &mut w);
    w.add_system(sky);

    w.run();
}
