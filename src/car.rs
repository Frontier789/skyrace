use glui::graphics::DrawShaderSelector;
use glui::mecs::{Component, DrawComponent, Entity, StaticWorld, System};
use glui::tools::mesh::{Mesh, MeshOnGPU};
use glui::tools::{Mat4, Uniform, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

pub struct CarSystem {
    body_mesh: MeshOnGPU,
    wheel_mesh: MeshOnGPU,
}

impl System for CarSystem {
    fn update(&mut self, delta_time: Duration, world: &mut StaticWorld) {
        let dt = delta_time.as_secs_f32();

        for body_entity in world.entities().iter() {
            let mut car_clone = None;

            if let Some(car) = world.component_mut::<CarComponent>(*body_entity) {
                if car.steer.abs() > 0.0001 {
                    let turn_angle = car.max_turn_angle * car.steer;
                    let turn_cirlce_radius = car.wheel_base / turn_angle.sin();
                    let angular_velocity = car.velocity.length() / turn_cirlce_radius;

                    car.dir = car.dir.rotate_y(angular_velocity * dt);
                    car.velocity = car.dir * car.velocity.length();
                }

                let g = 10.0;
                let car_weight = car.mass * g;

                let f_traction =
                    car.dir * 4500 * car.throttle - car.dir * car.mass * car.break_ * g;
                let f_drag = -car.c_drag * car.velocity * car.velocity.length();
                let f_rr = -car.c_rr * car.velocity;

                let f_long = f_traction + f_drag + f_rr;
                let a = f_long / car.mass;
                car.velocity += a * dt;

                let mut acceleration = a.dot(car.dir);

                if car.velocity.dot(car.dir) < 0.0 {
                    car.velocity = Vec3::zero();
                    acceleration = 0.0;
                }

                let weight_front =
                    0.5 * car_weight - car.cm_height / car.wheel_base * car.mass * acceleration;
                let weight_rear =
                    0.5 * car_weight + car.cm_height / car.wheel_base * car.mass * acceleration;

                car.wheel_turn += car.velocity.length() * dt / car.wheel_radius;

                car.rear_susp.apply_force(weight_rear, dt, car.mass);
                car.front_susp.apply_force(weight_front, dt, car.mass);

                car.pos += car.velocity * dt;
                car_clone = Some(car.clone());
            }

            if let Some(car) = car_clone {
                let car_h = (car.front_susp.length + car.rear_susp.length) / 2.0 + car.wheel_radius;
                let car_pitch =
                    (car.rear_susp.length - car.front_susp.length).atan2(car.wheel_base);

                let body_draw = world.component_mut::<DrawComponent>(*body_entity).unwrap();
                let look_at_mat = Mat4::look_at(
                    Vec3::origin(),
                    car.dir * Vec3::new(-1.0, 1.0, 1.0),
                    Vec3::new(0.0, 1.0, 0.0),
                );

                body_draw.model_matrix = Mat4::offset(car.pos + Vec3::new(0.0, car_h, 0.0))
                    * look_at_mat
                    * Mat4::rotate_x(car_pitch)
                    * Mat4::scale3(car.body_size / 2.0)
                    * Mat4::offset(Vec3::new(0.0, 1.0, 0.0));

                for (x, y, i) in [
                    (-1.0, -1.0, 0),
                    (-1.0, 1.0, 1),
                    (1.0, 1.0, 2),
                    (1.0, -1.0, 3),
                ]
                .iter()
                .copied()
                {
                    let wheel_draw = world
                        .component_mut::<DrawComponent>(car.wheels[i as usize])
                        .unwrap();

                    let angle = if y == -1.0 {
                        car.max_turn_angle * car.steer
                    } else {
                        0.0
                    };

                    wheel_draw.model_matrix = Mat4::offset(car.pos)
                        * look_at_mat
                        * Mat4::offset(Vec3::new(
                            car.body_size.x / 2.0 * x,
                            0.0,
                            car.wheel_base / 2.0 * y,
                        ))
                        * Mat4::rotate_y(PI / 2.0 + angle)
                        * Mat4::scale3(Vec3::new(
                            car.wheel_radius,
                            car.wheel_radius,
                            car.wheel_width,
                        ))
                        * Mat4::offset(Vec3::new(0.0, 1.0, 0.0))
                        * Mat4::rotate_z(car.wheel_turn);
                }
            }
        }
    }

    // fn receive(&mut self, msg: &Box<dyn Message>, world: &mut StaticWorld) {
    //     if let Some(SetSunDir(d)) = msg.downcast_ref() {
    //         let d = *d;
    //         world.with_component_mut(self.draw_entity, |c: &mut DrawComponent| {
    //             c.render_seq.command_mut(0).uniforms[0] = Uniform::Vector3("L".to_owned(), d);
    //         });
    //     }
    // }
}

// for entity in world.entities().iter() {
//     let mut is_car = false;
//     let mut car_pos = Vec3::zero();
//     let mut car_dir = Vec3::zero();
//     let mut car_h = 0.0;
//     let mut car_pitch = 0.0;
//     let mut car_size = Vec3::zero();
//
//     if let Some(car) = world.component_mut::<CarComponent>(*entity) {
//         if car.steer.abs() > 0.0001 {
//             let turn_angle = 30.0 / 180.0 * PI * car.steer;
//             let turn_cirlce_radius = car.wheel_base / turn_angle.sin();
//             let angular_velocity = car.velocity.length() / turn_cirlce_radius;
//
//             car.dir = car.dir.rotate_y(angular_velocity * dt);
//             car.velocity = car.dir * car.velocity.length();
//         }
//
//         let g = 10.0;
//         let car_weight = car.mass * g;
//
//         let f_traction =
//             car.dir * 4500 * car.throttle - car.dir * car.mass * car.break_ * g;
//         let f_drag = -car.c_drag * car.velocity * car.velocity.length();
//         let f_rr = -car.c_rr * car.velocity;
//
//         let f_long = f_traction + f_drag + f_rr;
//         let a = f_long / car.mass;
//         car.velocity += a * dt;
//
//         let mut acceleration = a.dot(car.dir);
//
//         if car.velocity.dot(car.dir) < 0.0 {
//             car.velocity = Vec3::zero();
//             acceleration = 0.0;
//         }
//
//         let weight_front =
//             0.5 * car_weight - car.cm_height / car.wheel_base * car.mass * acceleration;
//         let weight_rear =
//             0.5 * car_weight + car.cm_height / car.wheel_base * car.mass * acceleration;
//
//         car.rear_susp.apply_force(weight_rear, dt, car.mass);
//         car.front_susp.apply_force(weight_front, dt, car.mass);
//         car_h = (car.front_susp.length + car.rear_susp.length) / 2.0;
//         car_pitch = (car.rear_susp.length - car.front_susp.length).atan2(car.wheel_base);
//         car_size = car.body_size;
//
//         car.pos += car.velocity * dt;
//
//         is_car = true;
//         car_pos = car.pos;
//         car_dir = car.dir;
//     }
//
//     if is_car {
//         let draw = world.component_mut::<DrawComponent>(*entity).unwrap();
//         draw.model_matrix = Mat4::offset(car_pos + Vec3::new(0.0, car_h, 0.0))
//             * Mat4::look_at(
//                 Vec3::origin(),
//                 car_dir * Vec3::new(-1.0, 1.0, 1.0),
//                 Vec3::new(0.0, 1.0, 0.0),
//             )
//             * Mat4::rotate_x(car_pitch)
//             * Mat4::scale3(car_size / 2.0)
//             * Mat4::offset(Vec3::new(0.0, 1.0, 0.0));
//     }

impl CarSystem {
    pub fn new() -> CarSystem {
        let body = Mesh::unit_cube().upload_to_gpu();
        let wheel = Mesh::unit_cylinder(9).upload_to_gpu();

        CarSystem {
            body_mesh: body,
            wheel_mesh: wheel,
        }
    }

    fn create_wheel(&self, world: &mut StaticWorld) -> Entity {
        world.new_entity_with_component(DrawComponent::from_render_seq(
            self.wheel_mesh.non_owning_render_seq(
                DrawShaderSelector::DiffusePhong,
                vec![Uniform::Vector3(
                    "L".to_owned(),
                    Vec3::new(1.0, 0.3, 1.0).sgn(),
                )],
            ),
        ))
    }

    pub fn create_car(&self, world: &mut StaticWorld) -> Entity {
        let e = world.entity();
        world.add_component(
            e,
            DrawComponent::from_render_seq(self.body_mesh.non_owning_render_seq(
                DrawShaderSelector::DiffusePhong,
                vec![Uniform::Vector3(
                    "L".to_owned(),
                    Vec3::new(1.0, 0.3, 1.0).sgn(),
                )],
            )),
        );
        let wheels = [
            self.create_wheel(world),
            self.create_wheel(world),
            self.create_wheel(world),
            self.create_wheel(world),
        ];
        world.add_component(e, CarComponent::new(wheels));
        e
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Suspension {
    pub eq_length: f32,
    pub length: f32,
    pub velocity: f32,
    pub dampening: f32,
    pub stiffness: f32,
}

impl Suspension {
    pub fn new(eq_length: f32, dampening: f32, stiffness: f32) -> Suspension {
        Suspension {
            eq_length,
            dampening,
            stiffness,
            velocity: 0.0,
            length: eq_length,
        }
    }

    pub fn apply_force(&mut self, force: f32, time: f32, mass: f32) {
        let dx = self.eq_length - self.length;
        let net_force = -force + dx * self.stiffness - self.velocity * self.dampening;
        let acceleration = net_force / mass;

        self.velocity += acceleration * time;
        self.length += self.velocity * time;
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct CarComponent {
    pub pos: Vec3,
    pub dir: Vec3,
    pub velocity: Vec3,
    pub c_drag: f32,
    pub c_rr: f32,
    pub mass: f32,
    pub wheel_base: f32,
    pub wheel_friction_coef: f32,
    pub cm_height: f32,
    pub rear_susp: Suspension,
    pub front_susp: Suspension,
    pub angular_velocity: f32,
    pub body_size: Vec3,
    pub wheel_radius: f32,
    pub wheel_width: f32,
    pub max_turn_angle: f32,
    pub wheel_turn: f32,

    pub throttle: f32,
    pub break_: f32,
    pub steer: f32,

    pub wheels: [Entity; 4],
}

impl CarComponent {
    pub fn new(wheels: [Entity; 4]) -> CarComponent {
        CarComponent {
            pos: Vec3::origin(),
            dir: Vec3::new(1.0, 0.0, -1.0).sgn(),
            velocity: Vec3::zero(),
            angular_velocity: 0.0,
            c_drag: 1.0,
            c_rr: 30.0,
            mass: 1500.0,
            throttle: 0.0,
            break_: 0.0,
            wheel_base: 2.7,
            steer: 0.0,
            wheel_friction_coef: 1.0,
            cm_height: 0.8,
            front_susp: Suspension::new(0.35, 4500.0, 52000.0),
            rear_susp: Suspension::new(0.4, 4200.0, 34000.0),
            body_size: Vec3::new(1.9, 1.4, 4.2),
            wheel_radius: 0.45,
            wheel_width: 0.2,
            wheels,
            max_turn_angle: 30.0 / 180.0 * PI,
            wheel_turn: 0.0,
        }
    }
}
