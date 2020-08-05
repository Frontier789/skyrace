use crate::line_system::{LineSystem, LinesUpdate, SetLine};
use glui::graphics::DrawShaderSelector;
use glui::mecs::{Component, DrawComponent, Entity, StaticWorld, System};
use glui::tools::mesh::{Mesh, MeshOnGPU};
use glui::tools::{Mat4, Uniform, Vec2, Vec3};
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
            let msgs: Vec<SetLine> = vec![];
            let mut car_clone = None;

            if let Some(car) = world.component_mut::<CarComponent>(*body_entity) {
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
                let steer_angle = steer_angle / (1.0 + car.speed() / 40.0);

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

                if car.speed() < 6.0 {
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

                car_clone = Some(car.clone());
            }

            if let Some(car) = car_clone {
                for msg in msgs {
                    world.send_by_type::<LineSystem, _>(msg);
                }
                world.send_by_type::<LineSystem, _>(LinesUpdate {});

                let car_h =
                    (car.front_susp.length + car.rear_susp.length) / 2.0 + car.config.wheel_radius;
                let car_pitch =
                    (car.front_susp.length - car.rear_susp.length).atan2(car.config.wheel_base());

                let body_draw = world.component_mut::<DrawComponent>(*body_entity).unwrap();

                body_draw.model_matrix = Mat4::offset(car.pos3() + Vec3::new(0.0, car_h, 0.0))
                    * Mat4::rotate_y(-car.heading)
                    * Mat4::rotate_z(car_pitch)
                    * Mat4::scale3(car.config.body_size() / 2.0)
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

                    let angle = if y == 1.0 { car.wheel_turn() } else { 0.0 };

                    wheel_draw.model_matrix = Mat4::offset(car.pos3())
                        * Mat4::rotate_y(-car.heading)
                        * Mat4::offset(Vec3::new(
                            car.config.wheel_base() / 2.0 * y,
                            0.0,
                            (car.config.body_size().z / 2.0 + car.config.wheel_width) * x,
                        ))
                        * Mat4::rotate_y(-angle)
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
    }
}

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
                vec![Uniform::from(
                    "light_direction",
                    Vec3::new(1.0, 0.3, 1.0).sgn(),
                )],
            ),
        ))
    }

    pub fn create_car(&self, world: &mut StaticWorld, init_state: (f32, Vec2)) -> Entity {
        let e = world.entity();
        world.add_component(
            e,
            DrawComponent::from_render_seq(self.body_mesh.non_owning_render_seq(
                DrawShaderSelector::DiffusePhong,
                vec![Uniform::from(
                    "light_direction",
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
        world.add_component(e, CarComponent::new_stiff(wheels, init_state));
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
}

impl CarComponent {
    #[allow(dead_code)]
    pub fn new_loose(wheels: [Entity; 4]) -> Self {
        CarComponent {
            config: CarConfig {
                gravity: 10.0,
                mass: 1500.0,
                inertia_ratio: 1.0,
                width: 1.6,
                cg_to_front: 2.0,
                cg_to_rear: 2.0,
                cg_to_front_axle: 1.25,
                cg_to_rear_axle: 1.25,
                cg_height: 0.55,
                wheel_radius: 0.45,
                wheel_width: 0.15,
                tire_grip: 1.0,
                lock_grip: 0.7,
                engine_force: 8000.0,
                brake_force: 12000.0,
                hand_break_force: 12000.0 / 2.5,
                weight_transfer: 0.2,
                max_steer: 0.8,
                corner_stiffness_front: 5.0,
                corner_stiffness_rear: 5.2,
                air_resistance: 2.5,
                roll_resistance: 10.0,
                body_height: 1.4,
            },
            heading: 0.0,
            position: Vec2::zero(),
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
        }
    }

    fn new_stiff(wheels: [Entity; 4], init_state: (f32, Vec2)) -> Self {
        CarComponent {
            config: CarConfig {
                gravity: 10.0,
                mass: 1500.0,
                inertia_ratio: 0.3,
                width: 1.6,
                cg_to_front: 2.0,
                cg_to_rear: 2.0,
                cg_to_front_axle: 1.25,
                cg_to_rear_axle: 1.25,
                cg_height: 0.55,
                wheel_radius: 0.45,
                wheel_width: 0.15,
                tire_grip: 5.0,
                lock_grip: 0.7,
                engine_force: 8000.0,
                brake_force: 12000.0,
                hand_break_force: 12000.0 / 2.5,
                weight_transfer: 0.2,
                max_steer: 0.35,
                corner_stiffness_front: 10.0,
                corner_stiffness_rear: 10.2,
                air_resistance: 2.5,
                roll_resistance: 10.0,
                body_height: 1.4,
            },
            heading: init_state.0,
            position: init_state.1,
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
        }
    }

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
        Vec3::new(self.position.x, 0.0, self.position.y)
    }

    pub fn wheel_turn(&self) -> f32 {
        self.config.max_steer * self.steer
    }
}
