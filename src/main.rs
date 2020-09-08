#![feature(clamp)]

mod ai_driver;
mod cacti;
mod camera_on_car;
mod playback_driver;
mod sounds;
mod sun_mover;
mod terrain;
mod track;
mod utilities;
mod wheel_mark;

mod car;
mod car_driver;
mod gui;
mod line_system;
mod sky;

extern crate glui;
extern crate glui_proc;
extern crate noise;

use crate::ai_driver::AiDriver;
use crate::camera_on_car::CamFollowCar;
use crate::car::CarSystem;
use crate::car_driver::CarDriver;
use crate::gui::Gui;
use crate::line_system::LineSystem;
// use crate::playback_driver::{CarDriveState, PlaybackDriver};
use crate::sky::Sky;
use crate::sun_mover::SunMover;
use crate::terrain::Terrain;
use crate::track::Track;
use glui::mecs::World;
use glui::mecs::*;
// use glui::tools::serde_tools::SerdeJsonQuick;
use crate::cacti::Cacti;
use crate::sounds::Sounds;
use crate::wheel_mark::WheelMark;
use glui::tools::*;
use rand::distributions::{Distribution, Uniform};
use std::env;

fn main() {
    let mut follow_ai = true;
    let mut muted = false;
    for arg in env::args() {
        if arg == "-race" {
            follow_ai = !follow_ai;
        }
        if arg == "-mute" {
            muted = !muted;
        }
    }

    let mut w: World = World::new_win(Vec2::new(1024.0, 768.0), "Skyracer", Vec3::grey(0.1));

    let ds = DrawSystem::new(&mut w, NoController {});
    let camera_entity = ds.camera_entity;
    ds.camera_mut(w.as_static_mut()).params.zfar = 6000.0;

    w.add_system(ds);

    let carsys = CarSystem::new();
    let linesys = LineSystem::new(w.as_static_mut());
    w.add_system(linesys);

    let terrain = Terrain::new(w.as_static_mut());
    let cacti = Cacti::new(w.as_static_mut(), terrain.height_tex());
    w.add_system(terrain);
    w.add_system(cacti);

    // if let Ok(path) = Vec::<CarDriveState>::load_json("path.json") {
    //     let car = carsys.create_car(w.as_static_mut(), (0.0, Vec2::zero()), Vec4::grey(0.4));
    //     let driver = PlaybackDriver::new(car, path);
    //     w.add_system(driver);
    // }

    let sun_dir = Vec3::new(-1.0, 0.3, -1.0).sgn();

    let track = Track::new(w.as_static_mut()).expect("Failed to init track!");

    let mut cars = vec![];

    if !follow_ai {
        let state = (
            -5.39,
            Vec2::new(473.0, 184.0) * 0.8 + Vec2::new(-0.77907276, 0.6269335) * (2.2 * 1.5 - 1.0)
                - Vec2::new(0.6269335, 0.77907276) * 9.0 * 2.0,
        );
        let car = carsys.create_car(
            w.as_static_mut(),
            state,
            Vec4::new(0.5333, 0.2014, 0.0314, 1.0),
            0.05,
        );
        let driver = CarDriver::new(car);
        w.add_system(driver);

        cars.push(car);
    }

    let mut rng = rand::thread_rng();
    let distr = Uniform::new(0.0, 1.0);
    for j in [-5.4, -2.2, 2.2, 5.4].iter() {
        for i in [0, 2, 4].iter() {
            let state = (
                -5.39,
                Vec2::new(473.0, 184.0) * 0.8
                    + Vec2::new(-0.77907276, 0.6269335) * (*j * 1.5 - 1.0)
                    - Vec2::new(0.6269335, 0.77907276) * 9.0 * *i,
            );
            if *i != 2 || *j != 2.2 || follow_ai {
                let car = carsys.create_car(
                    w.as_static_mut(),
                    state,
                    Vec4::new(
                        distr.sample(&mut rng),
                        distr.sample(&mut rng),
                        distr.sample(&mut rng),
                        1.0,
                    ),
                    0.1,
                );
                let driver = AiDriver::new(car, j / 2.0).expect("Failed to read path!");
                w.add_system(driver);
                cars.push(car);
            }
        }
    }

    let sounds = Sounds::new(cars.clone(), camera_entity, muted);
    w.add_system(sounds);

    let marks = WheelMark::new(cars.clone(), w.as_static_mut());
    w.add_system(marks);

    w.add_gui(Gui::from_car(cars[0]));
    let follower = CamFollowCar::new(cars, camera_entity, false, w.as_static_mut());
    w.add_system(follower);

    let sky = Sky::new(sun_dir, &mut w);

    w.add_system(sky);
    w.add_system(track);
    w.add_system(carsys);

    let sun_dir_setter = SunMover::new();
    w.add_system(sun_dir_setter);

    w.run();

    // let cam = &mut w
    //     .component_mut::<DataComponent<Camera>>(camera_entity)
    //     .unwrap()
    //     .data;
    //
    // cam.params.spatial.save_json("cam.json").unwrap();
    //
    // let car = &mut w.component_mut::<CarComponent>(car).unwrap();
    //
    // car.spatial_state().save_json("car_state.json").unwrap();
}
