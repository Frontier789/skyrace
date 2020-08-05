#![feature(clamp)]

mod camera_on_car;
mod playback_driver;
mod sun_mover;
mod track;

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
use crate::sun_mover::SunMover;
use crate::track::Track;
use glui::mecs::World;
use glui::mecs::*;
use glui::tools::serde_tools::SerdeJsonQuick;
use glui::tools::*;

fn main() {
    let mut w: World = World::new_win(Vec2::new(1024.0, 768.0), "Skyracer", Vec3::grey(0.1));

    let ds = DrawSystem::new(&mut w, NoController {});
    let camera_entity = ds.camera_entity;
    ds.camera_mut(w.as_static_mut()).params.zfar = 6000.0;

    w.add_system(ds);

    let carsys = CarSystem::new();
    let linesys = LineSystem::new(w.as_static_mut());
    let ground = Ground::new(w.as_static_mut());
    w.add_system(ground);

    if let Ok(path) = Vec::<CarDriveState>::load_json("path.json") {
        let car = carsys.create_car(w.as_static_mut(), (0.0, Vec2::zero()));
        let driver = PlaybackDriver::new(car, path);
        w.add_system(driver);
    }

    let sun_dir = Vec3::new(-1.0, 0.3, -1.0).sgn();

    let track = Track::new(w.as_static_mut()).unwrap();

    let car_state = <(f32, Vec2)>::load_json("car_state.json").unwrap_or_default();
    let car = carsys.create_car(w.as_static_mut(), car_state);
    let driver = CarDriver::new(car);
    w.add_system(driver);

    let follower = CamFollowCar::new(car, camera_entity, true, w.as_static_mut());
    w.add_system(follower);
    w.add_gui(Gui::from_car(car));

    let sky = Sky::new(sun_dir, &mut w);

    w.add_system(sky);
    w.add_system(track);
    w.add_system(linesys);
    w.add_system(carsys);

    let sun_dir_setter = SunMover::new();
    w.add_system(sun_dir_setter);

    w.run();

    let cam = &mut w
        .component_mut::<DataComponent<Camera>>(camera_entity)
        .unwrap()
        .data;

    cam.params.spatial.save_json("cam.json").unwrap();

    let car = &mut w.component_mut::<CarComponent>(car).unwrap();

    car.spatial_state().save_json("car_state.json").unwrap();
}
