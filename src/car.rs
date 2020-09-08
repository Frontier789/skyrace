extern crate assimp;
extern crate rand;

use self::assimp::{aiImportFileToMesh, aiImportFileToMeshes};
#[allow(deprecated)]
use self::rand::distributions::{Distribution, Normal};
// use crate::line_system::{DelLine, LineDesc, LineSystem, LinesUpdate, SetLine};
use glui::graphics::{DrawShaderSelector, RenderCommand, RenderSequence};
use glui::mecs::{BodyComponent, Component, DrawComponent, Entity, StaticWorld, System};
use glui::tools::mesh::{Mesh, MeshOnGPU};
use glui::tools::{
    Buffer, DrawMode, Mat4, Rect, RgbaTexture, Uniform, Vec2, Vec3, Vec4, VertexArray,
};
use std::f32::consts::PI;
use std::time::Duration;

struct CarBody {
    meshes: Vec<MeshOnGPU>,
}

pub struct CarSystem {
    body_mesh: CarBody,
    wheel_mesh: MeshOnGPU,
    shadow: RgbaTexture,
}

impl System for CarSystem {
    fn update(&mut self, delta_time: Duration, world: &mut StaticWorld) {
        let dt = delta_time.as_secs_f32();

        let car_entities = world.entities_having_component::<CarComponent>();

        for i in 0..car_entities.len() {
            let body_entity = car_entities[i];
            let mut car_clone = None;

            if let Some(car) = world.component_mut::<CarComponent>(body_entity) {
                // println!(
                //     "throttle: {}, brake: {}, steer: {}",
                //     car.throttle, car.brake, car.steer
                // );

                // init config
                let cfg = &car.config;
                let inertia = cfg.mass * cfg.inertia_ratio;
                let wheel_base = cfg.wheel_base();
                let axle_weight_ratio_front = cfg.cg_to_rear_axle / wheel_base;
                let axle_weight_ratio_rear = cfg.cg_to_front_axle / wheel_base;
                let steer_angle = cfg.max_steer * car.steer;
                let steer_angle = steer_angle / (1.0 + car.speed() / 60.0);

                // local base
                let dir = car.dir();
                let right = car.right();

                // velocity and acceleration in local coordinates
                let vel_long = car.velocity.dot(dir);
                let vel_lat = car.velocity.dot(right);
                let acc_long = car.acceleration.dot(dir);

                // weight on axles
                let axle_weight_front = cfg.mass
                    * (axle_weight_ratio_front * cfg.gravity
                        - cfg.weight_transfer * acc_long * cfg.cg_height / wheel_base);
                let axle_weight_rear = cfg.mass
                    * (axle_weight_ratio_rear * cfg.gravity
                        + cfg.weight_transfer * acc_long * cfg.cg_height / wheel_base);

                // velocity of wheels from angular velocity
                let vel_from_angular_front = cfg.cg_to_front_axle * car.angular_velocity;
                let vel_from_angular_rear = -cfg.cg_to_rear_axle * car.angular_velocity;

                // slip angles
                let mut slip_angle_front = (vel_lat + vel_from_angular_front).atan2(vel_long.abs())
                    - vel_long.signum() * steer_angle;
                let mut slip_angle_rear = (vel_lat + vel_from_angular_rear).atan2(vel_long.abs());

                // print!("{}, {} -> ", slip_angle_front, slip_angle_rear);
                if slip_angle_front > PI / 2.0 {
                    slip_angle_front = PI - slip_angle_front;
                }
                if slip_angle_rear > PI / 2.0 {
                    slip_angle_rear = PI - slip_angle_rear;
                }
                if slip_angle_front < -PI / 2.0 {
                    slip_angle_front = -PI - slip_angle_front;
                }
                if slip_angle_rear < -PI / 2.0 {
                    slip_angle_rear = -PI - slip_angle_rear;
                }
                // println!("{}, {}", slip_angle_front, slip_angle_rear);

                let tire_grip_front = cfg.tire_grip;
                let tire_grip_rear = cfg.tire_grip; // todo add support for hand break

                let f_friction_front = (-cfg.corner_stiffness_front * slip_angle_front)
                    .clamp(-tire_grip_front, tire_grip_front)
                    * axle_weight_front;
                let f_friction_rear = (-cfg.corner_stiffness_rear * slip_angle_rear)
                    .clamp(-tire_grip_rear, tire_grip_rear)
                    * axle_weight_rear;

                // brake and throttle forces
                let f_brake = car.brake * cfg.brake_force; // todo hand break
                let f_throttle = car.throttle * cfg.engine_force;

                // brake and throttle local force
                let f_traction = f_throttle - f_brake * vel_long.signum();

                let f_drag_long = -cfg.roll_resistance * vel_long
                    - cfg.air_resistance * vel_long * vel_long.abs();
                let f_drag_lat =
                    -cfg.roll_resistance * vel_lat - cfg.air_resistance * vel_lat * vel_lat.abs();

                // total local force
                let f_tot_long = f_drag_long - steer_angle.sin() * f_friction_front + f_traction;
                let f_tot_lat = f_drag_lat + steer_angle.cos() * f_friction_front + f_friction_rear;

                // local acceleration
                let a_long = f_tot_long / cfg.mass;
                let a_lat = f_tot_lat / cfg.mass;

                // acceleration in world coordinates
                let a = a_long * dir + a_lat * right;

                car.front_susp.apply_force(axle_weight_front, dt, cfg.mass);
                car.rear_susp.apply_force(axle_weight_rear, dt, cfg.mass);

                if car.speed() < 14.0 {
                    car.angular_velocity = car.speed() / cfg.wheel_base() * steer_angle.sin();

                    let a = (f_drag_long + f_traction) / cfg.mass;

                    car.acceleration = a * dir;
                    car.velocity = (car.speed() + a * dt) * dir;

                    if car.speed() < 0.2 && car.throttle == 0.0 {
                        car.velocity = Vec2::zero();
                        // car.acceleration = Vec2::zero();
                    }

                    car.heading += car.angular_velocity * dt;

                    car.position += car.velocity * dt;
                } else {
                    car.acceleration = a;
                    car.velocity += car.acceleration * dt;

                    // rotational forces
                    let body_torque = f_friction_front * cfg.cg_to_front_axle
                        - f_friction_rear * cfg.cg_to_rear_axle;

                    let angular_acceleration = body_torque / inertia;

                    car.angular_velocity += angular_acceleration * dt;
                    car.heading += car.angular_velocity * dt;

                    car.position += car.velocity * dt;
                }

                car.wheel_roll += car.speed() / cfg.wheel_radius * dt;

                car.terrain_height = 0.0;

                car_clone = Some(car.clone());
            }
            //
            // let bs1 = car_clone.unwrap().config.body_size();
            // let p1 = car_clone.unwrap().position;
            // let d1 = car_clone.unwrap().dir() * bs1.x / 2.0;
            // let r1 = car_clone.unwrap().right() * bs1.z / 2.0;
            //
            // world.send_by_type::<LineSystem, _>(SetLine(
            //     format!("bound_line {:?},0", i),
            //     LineDesc {
            //         color: Vec4::RED,
            //         a: Vec3::from_vec2(p1 + d1 + r1, 2.0).xzy(),
            //         b: Vec3::from_vec2(p1 + d1 - r1, 2.0).xzy(),
            //     },
            // ));
            // world.send_by_type::<LineSystem, _>(SetLine(
            //     format!("bound_line {:?},1", i),
            //     LineDesc {
            //         color: Vec4::RED,
            //         a: Vec3::from_vec2(p1 + d1 - r1, 2.0).xzy(),
            //         b: Vec3::from_vec2(p1 - d1 - r1, 2.0).xzy(),
            //     },
            // ));
            // world.send_by_type::<LineSystem, _>(SetLine(
            //     format!("bound_line {:?},2", i),
            //     LineDesc {
            //         color: Vec4::RED,
            //         a: Vec3::from_vec2(p1 - d1 - r1, 2.0).xzy(),
            //         b: Vec3::from_vec2(p1 - d1 + r1, 2.0).xzy(),
            //     },
            // ));
            // world.send_by_type::<LineSystem, _>(SetLine(
            //     format!("bound_line {:?},3", i),
            //     LineDesc {
            //         color: Vec4::RED,
            //         a: Vec3::from_vec2(p1 - d1 + r1, 2.0).xzy(),
            //         b: Vec3::from_vec2(p1 + d1 + r1, 2.0).xzy(),
            //     },
            // ));

            // collision update
            // for j in i + 1..car_entities.len() {
            //     let car = world
            //         .component_mut::<CarComponent>(car_entities[j])
            //         .unwrap();
            //     let bs2 = car.config.body_size();
            //     let p2 = car.position;
            //     let d2 = car.dir() * bs2.x / 2.0;
            //     let r2 = car.right() * bs2.z / 2.0;
            //
            //     for (cx, cy) in [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)].iter() {
            //         for (o, corner, p, d, r, k) in [
            //             (p1, p1 + *cx * d1 + *cy * r1, p2, d2, r2, 0),
            //             (p2, p2 + *cx * d2 + *cy * r2, p1, d1, r1, 1),
            //         ]
            //         .iter()
            //         {
            //             if let Some(n) = point_in_rect(*corner, *o - *corner, *p, *d, *r) {
            //                 if let Some(car) = world.component_mut::<CarComponent>(car_entities[j])
            //                 {
            //                     car.position += n / 2.0;
            //                 }
            //                 if let Some(car) = world.component_mut::<CarComponent>(car_entities[i])
            //                 {
            //                     car.position -= n / 2.0;
            //                 }
            //                 world.send_by_type::<LineSystem, _>(SetLine(
            //                     format!("coll_line {:?},{:?},{}", (cx, cy), (i, j), k),
            //                     LineDesc {
            //                         color: Vec4::GREEN,
            //                         a: Vec3::from_vec2(*corner, 2.0).xzy(),
            //                         b: Vec3::from_vec2(*corner + n, 2.0).xzy(),
            //                     },
            //                 ));
            //             } else {
            //                 world.send_by_type::<LineSystem, _>(DelLine(format!(
            //                     "coll_line {:?},{:?},{}",
            //                     (cx, cy),
            //                     (i, j),
            //                     k
            //                 )));
            //             }
            //         }
            //     }
            // }

            // draw update
            if let Some(car) = car_clone {
                let car_h =
                    (car.front_susp.length + car.rear_susp.length) / 2.0 + car.config.wheel_radius;
                let car_pitch =
                    (car.front_susp.length - car.rear_susp.length).atan2(car.config.wheel_base());

                let ori_scale =
                    Mat4::rotate_y(-car.heading) * Mat4::scale3(car.config.body_size() / 2.0);

                let ori_scale_bob = Mat4::rotate_y(-car.heading)
                    * Mat4::rotate_z(car_pitch)
                    * Mat4::scale3(car.config.body_size() / 2.0);

                let body_draw = world.component_mut::<DrawComponent>(body_entity).unwrap();

                body_draw.model_matrix = Mat4::offset(car.pos3() + Vec3::new(0.0, car_h, 0.0))
                    * ori_scale_bob
                    * Mat4::offset(Vec3::new(0.0, 1.0, 0.0));

                let mut light_dir = Vec3::zero();

                if let Uniform::Vector3(_, dir) = body_draw.render_seq.command(0).uniforms[0] {
                    light_dir = dir * Vec3::new(1.0, 0.0, 1.0);
                }

                let shadow_draw = world.component_mut::<DrawComponent>(car.shadow).unwrap();

                let depth = 0.02 + i as f32 * 0.0035;
                let center = car.pos3() + Vec3::new(0.0, depth, 0.0);
                shadow_draw.model_matrix = Mat4::offset(center - light_dir * 0.1) * ori_scale;

                let shadow_body = world.component_mut::<BodyComponent>(car.shadow).unwrap();
                shadow_body.center.z = -(i as f32);

                let wheel_dist = car.config.body_size().z / 2.0 - car.config.wheel_width;

                for (x, z, i) in [
                    (car.config.cg_to_rear_axle, -wheel_dist, 0),
                    (car.config.cg_to_rear_axle, wheel_dist, 1),
                    (-car.config.cg_to_front_axle, wheel_dist, 2),
                    (-car.config.cg_to_front_axle, -wheel_dist, 3),
                ]
                .iter()
                .copied()
                {
                    let wheel_draw = world
                        .component_mut::<DrawComponent>(car.wheels[i as usize])
                        .unwrap();

                    let turn_angle = if i == 0 || i == 1 {
                        car.wheel_turn()
                    } else {
                        0.0
                    } + if i == 1 || i == 2 { PI } else { 0.0 };

                    wheel_draw.model_matrix = Mat4::offset(car.pos3())
                        * Mat4::rotate_y(-car.heading)
                        * Mat4::offset(Vec3::new(x, 0.0, z))
                        * Mat4::rotate_y(-turn_angle)
                        * Mat4::scale3(Vec3::new(
                            car.config.wheel_radius,
                            car.config.wheel_radius,
                            car.config.wheel_width,
                        ))
                        * Mat4::offset(Vec3::new(0.0, 1.0, 0.0))
                        * Mat4::rotate_z(-car.wheel_roll);
                }
            }
        }
        // world.send_by_type::<LineSystem, _>(LinesUpdate {});
    }
}
//
// fn point_in_rect(p: Vec2, _po: Vec2, o: Vec2, d: Vec2, r: Vec2) -> Option<Vec2> {
//     let x = (p - o).dot(d.sgn()) / d.length();
//     let y = (p - o).dot(r.sgn()) / r.length();
//
//     if x.abs().max(y.abs()) <= 1.0 {
//         let proj0 = 1.0 * d + y * r + o - p;
//         let proj1 = -1.0 * d + y * r + o - p;
//         let proj2 = x * d + 1.0 * r + o - p;
//         let proj3 = x * d - 1.0 * r + o - p;
//         let c0 = proj0.length();
//         let c1 = proj1.length();
//         let c2 = proj2.length();
//         let c3 = proj3.length();
//         let cm = c0.min(c1.min(c2.min(c3)));
//         Some(if c0 == cm {
//             proj0
//         } else if c1 == cm {
//             proj1
//         } else if c2 == cm {
//             proj2
//         } else {
//             proj3
//         })
//     } else {
//         None
//     }
// }

impl CarSystem {
    pub fn new() -> CarSystem {
        CarSystem {
            body_mesh: Self::load_body(),
            wheel_mesh: Self::load_wheel(),
            shadow: RgbaTexture::from_file("images/shadow.png").unwrap_or(RgbaTexture::unit()),
        }
    }

    fn load_wheel() -> MeshOnGPU {
        match aiImportFileToMesh("models/wheel_boarded.obj") {
            Some(mesh) => mesh,
            None => Mesh::unit_cylinder(9),
        }
        .upload_to_gpu()
    }
    fn load_body() -> CarBody {
        match aiImportFileToMeshes("models/body_low.obj") {
            Some(meshes) => {
                let mut aabb = meshes[0].aabb();
                for i in 1..meshes.len() {
                    aabb = meshes[i].extend_aabb(aabb);
                }
                CarBody {
                    meshes: meshes
                        .into_iter()
                        .map(|m| m.fit_into_aabb_into_unit_cube(aabb).upload_to_gpu())
                        .collect(),
                }
            }
            None => CarBody {
                meshes: vec![Mesh::unit_cube().upload_to_gpu()],
            },
        }
    }

    fn create_wheel(&self, world: &mut StaticWorld) -> Entity {
        world.new_entity_with_component(DrawComponent::from_render_seq(
            self.wheel_mesh.non_owning_render_seq(
                DrawShaderSelector::Phong,
                vec![
                    Uniform::from("light_direction", Vec3::new(1.0, 0.3, 1.0).sgn()),
                    Uniform::from("Kd", Vec3::grey(0.23)),
                    Uniform::from("Ka", Vec3::grey(0.1)),
                    Uniform::from("Ks", Vec3::grey(0.2)),
                    Uniform::from("Ns", 9.0),
                ],
            ),
        ))
    }
    fn all_body_render_seq(&self, primary_color: Vec4) -> RenderSequence {
        let mut rs = RenderSequence::new();
        let shader = DrawShaderSelector::Phong;
        let secondary_color = primary_color.yzxw();
        let colors = vec![
            primary_color * 0.8 + secondary_color, // Vec4::new(0.0235, 0.5294, 0.4431, 1.0),
            primary_color,                         // Vec4::new(0.0314, 0.0314, 0.5333, 1.0),
            Vec4::new(0.6039, 0.7255, 0.8980, 1.0),
            Vec4::WHITE - (primary_color + secondary_color) * 1.4, // Vec4::new(0.8784, 0.3373, 0.3373, 1.0),
            Vec4::WHITE - (primary_color * 2.0 + secondary_color * 0.6), // Vec4::new(0.5529, 0.0275, 0.2275, 1.0),
            Vec4::new(0.0000, 0.0000, 0.0000, 1.0),
            primary_color * 0.7 + Vec4::WHITE * 0.3, // Vec4::new(0.3333, 0.1098, 0.6941, 1.0),
            Vec4::new(0.6039, 0.8431, 0.8980, 1.0),
        ];
        let ns = vec![
            9.0,   // handle
            300.0, // body
            30.0,  // mirrors
            30.0,  // brand
            9.0,   // grid
            9.0,   // front_lamp
            9.0,   // back_lamp
            50.0,  // exhaust
        ];

        for i in 0..self.body_mesh.meshes.len() {
            let c = colors[i].rgb();
            let uniforms = vec![
                Uniform::from("light_direction", Vec3::new(1.0, 0.3, 1.0).sgn()),
                Uniform::from("Ka", c / 4.0),
                Uniform::from("Kd", c),
                Uniform::from("Ks", c * 0.3),
                Uniform::from("Ns", ns[i]),
            ];
            rs.add_command(self.body_mesh.meshes[i].as_render_command(shader.clone(), uniforms));
        }

        rs
    }

    fn shadow_rs(&self) -> RenderSequence {
        let pts = Rect::from_min_max(Vec2::new(-2.0, -2.0), Vec2::new(2.0, 2.0)).triangulate_3d();
        let pts = pts.into_iter().map(|p| p.xzy()).collect::<Vec<Vec3>>();
        let pbuf = Buffer::from_vec(&pts);
        let cbuf = Buffer::from_vec(&vec![Vec4::WHITE; 6]);
        let tbuf = Buffer::from_vec(&Rect::unit().triangulate());

        let mut vao = VertexArray::new();
        vao.attrib_buffer(0, &pbuf);
        vao.attrib_buffer(1, &cbuf);
        vao.attrib_buffer(2, &tbuf);

        let mut rs = RenderSequence::new();

        rs.add_buffer(pbuf.into_base_type());
        rs.add_buffer(cbuf.into_base_type());
        rs.add_buffer(tbuf.into_base_type());

        rs.add_command(RenderCommand {
            vao,
            mode: DrawMode::Triangles,
            shader: DrawShaderSelector::Textured,
            uniforms: vec![Uniform::from("tex", &self.shadow)],
            transparent: true,
            instances: 1,
            wireframe: false,
        });

        rs
    }

    pub fn create_car(
        &self,
        world: &mut StaticWorld,
        init_state: (f32, Vec2),
        color: Vec4,
        randomness: f32,
    ) -> Entity {
        let e = world.entity();
        world.add_component(
            e,
            DrawComponent::from_render_seq(self.all_body_render_seq(color)),
        );
        let wheels = [
            self.create_wheel(world),
            self.create_wheel(world),
            self.create_wheel(world),
            self.create_wheel(world),
        ];
        let shadow_entity = world.entity();
        world.add_component(
            shadow_entity,
            DrawComponent::from_render_seq(self.shadow_rs()),
        );
        world.add_component(
            shadow_entity,
            BodyComponent {
                center: Default::default(),
            },
        );
        world.add_component(
            e,
            CarComponent::new_stiff(wheels, shadow_entity, init_state, randomness),
        );
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

#[derive(Debug, Copy, Clone)]
pub struct CarConfig {
    pub gravity: f32,                // m/s^2
    pub mass: f32,                   // kg
    pub inertia_ratio: f32,          // mass * ratio = inertia
    pub width: f32,                  // width of car body (m)
    pub cg_to_front: f32,            // center of gravity to front (m)
    pub cg_to_rear: f32,             // center of gravity to rear (m)
    pub cg_to_front_axle: f32,       // center of gravity to front axle (m)
    pub cg_to_rear_axle: f32,        // center of gravity to rear axle (m)
    pub cg_height: f32,              // height of CG above axles (m)
    pub wheel_radius: f32,           // radius of wheels (m)
    pub wheel_width: f32,            // width of wheels (m)
    pub tire_grip: f32,              // grip of tires (ratio)
    pub lock_grip: f32,              // percentage of grip available when wheel is locked
    pub engine_force: f32,           // force exerted by the engine on the tires (N)
    pub brake_force: f32,            // breaking force (N)
    pub hand_break_force: f32,       // hand break force (N)
    pub weight_transfer: f32,        // how much weight is transferred during acceleration (ratio)
    pub max_steer: f32,              // maximum steering angle (rad)
    pub corner_stiffness_front: f32, // cornering stiffness of front wheels (N/rad)
    pub corner_stiffness_rear: f32,  // cornering stiffness of rear wheels (N/rad)
    pub air_resistance: f32,         // unitless
    pub roll_resistance: f32,        // unitless
    pub body_height: f32,            // m
}

impl CarConfig {
    pub fn wheel_base(&self) -> f32 {
        self.cg_to_rear_axle + self.cg_to_front_axle
    }
    pub fn body_size(&self) -> Vec3 {
        Vec3::new(
            self.cg_to_front + self.cg_to_rear,
            self.body_height,
            self.width,
        )
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct CarComponent {
    pub config: CarConfig,
    pub heading: f32,          // direction of car (rad)
    pub position: Vec2,        // world coordinates
    pub terrain_height: f32,   // height of the terrain below the car
    pub velocity: Vec2,        // world coordinates
    pub acceleration: Vec2,    // world coordinates
    pub angular_velocity: f32, // rad/s
    pub wheel_roll: f32,       // rad

    pub steer: f32,    // in [-1,1]
    pub throttle: f32, // in [0,1]
    pub brake: f32,    // in [0,1]

    pub rear_susp: Suspension,
    pub front_susp: Suspension,
    pub wheels: [Entity; 4],
    pub shadow: Entity,
}

impl CarComponent {
    fn new_stiff(
        wheels: [Entity; 4],
        shadow: Entity,
        init_state: (f32, Vec2),
        randomness: f32,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let mut rngs = rand::thread_rng();
        let normal = Normal::new(1.0, randomness as f64);
        let normals = Normal::new(1.0, randomness as f64 * 0.1);
        let mut rnd = || normal.sample(&mut rng) as f32;
        let mut rnds = || normals.sample(&mut rngs) as f32;
        let scale = 1.4 * rnds();
        CarComponent {
            config: CarConfig {
                gravity: 10.0 * rnds(),
                mass: 1500.0 * rnd(),
                inertia_ratio: 0.3 * rnds(),
                width: 1.8 * scale,
                cg_to_front: 2.04 * scale,
                cg_to_rear: 2.04 * scale,
                cg_to_front_axle: 1.13 * scale,
                cg_to_rear_axle: 1.29 * scale,
                cg_height: 0.55 * rnd(),
                wheel_radius: 0.45 * rnds(),
                wheel_width: 0.15 * rnds(),
                tire_grip: 5.0 * rnd(),
                lock_grip: 0.7 * rnd(),
                engine_force: 8000.0 * rnd(),
                brake_force: 12000.0 * rnd(),
                hand_break_force: 12000.0 / 2.5 * rnd(),
                weight_transfer: 0.2 * rnd(),
                max_steer: 0.7 * rnd(),
                corner_stiffness_front: 14.0 * rnd(),
                corner_stiffness_rear: 14.2 * rnd(),
                air_resistance: 2.5 * rnd(),
                roll_resistance: 10.0 * rnd(),
                body_height: 1.0 * scale,
            },
            heading: init_state.0,
            position: init_state.1,
            terrain_height: 0.0,
            velocity: Vec2::zero(),
            acceleration: Vec2::zero(),
            angular_velocity: 0.0,
            steer: 0.0,
            throttle: 0.0,
            brake: 0.0,
            front_susp: Suspension::new(0.0, 2500.0, 42000.0),
            rear_susp: Suspension::new(0.05, 2200.0, 34000.0),
            wheels,
            wheel_roll: 0.0,
            shadow,
        }
    }

    #[allow(dead_code)]
    pub fn spatial_state(&self) -> (f32, Vec2) {
        (self.heading, self.position)
    }

    pub fn speed(&self) -> f32 {
        self.velocity.length()
    }

    pub fn dir(&self) -> Vec2 {
        Vec2::pol(1.0, self.heading)
    }

    pub fn dir3(&self) -> Vec3 {
        let d = Vec2::pol(1.0, self.heading);
        Vec3::new(d.x, 0.0, d.y)
    }

    pub fn right(&self) -> Vec2 {
        self.dir().perp()
    }

    pub fn right3(&self) -> Vec3 {
        let r = self.dir().perp();
        Vec3::new(r.x, 0.0, r.y)
    }

    pub fn pos3(&self) -> Vec3 {
        Vec3::new(self.position.x, self.terrain_height, self.position.y)
    }

    pub fn wheel_turn(&self) -> f32 {
        self.config.max_steer * self.steer
    }
}
