use glui::mecs::{DrawComponent, System};
use glui::mecs::{GlutinKey, StaticWorld};
use glui::tools::{Uniform, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

#[derive(Debug)]
pub struct SunMover {
    time: f32,
}

impl SunMover {
    pub fn new() -> SunMover {
        SunMover { time: 0.0 }
    }
}

impl System for SunMover {
    fn update(&mut self, delta_time: Duration, world: &mut StaticWorld) {
        self.time += delta_time.as_secs_f32()
            * if world.is_key_pressed(GlutinKey::O) {
                2.0
            } else {
                0.002
            };
        let t = self.time;

        let dir = Vec3::pol(1.0, (t.sin() + 1.0) / 2.0 * 0.6, t - PI / 2.0).xzy();

        for (_e, comp) in world.entities_with_component_mut::<DrawComponent>() {
            for cmd in comp.render_seq.command_iter_mut() {
                for uniform in cmd.uniforms.iter_mut() {
                    if let Uniform::Vector3(name, _vec) = uniform {
                        if name == "light_direction" {
                            *uniform = Uniform::from(name, dir);
                        }
                    }
                }
            }
        }
    }
}
