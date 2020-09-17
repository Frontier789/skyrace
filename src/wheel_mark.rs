use crate::car::CarComponent;
use glui::graphics::{DrawShaderSelector, RenderCommand, RenderSequence};
use glui::mecs::*;
use glui::tools::{Buffer, DrawMode, RgbaTexture, Uniform, Vec2, Vec3, Vec4, VertexArray};
use std::time::Duration;

pub struct WheelMark {
    cars: Vec<Entity>,
    prev_pos: Vec<[Vec3; 4]>,
    pos_buf: Buffer<Vec3>,
    clr_buf: Buffer<Vec4>,
    tpt_buf: Buffer<Vec2>,
    _tex: RgbaTexture,
    next_id: usize,
    size: usize,
}

impl System for WheelMark {
    fn update(&mut self, _delta_time: Duration, world: &mut StaticWorld) {
        for j in 0..self.cars.len() {
            let car = world.component::<CarComponent>(self.cars[j]).unwrap();
            let p = car.pos3();
            let f = car.dir3();
            let r = car.right3();
            let wheel_p = [
                p + car.config.cg_to_front_axle * f + r * car.config.width / 2.0,
                p - car.config.cg_to_rear_axle * f + r * car.config.width / 2.0,
                p + car.config.cg_to_front_axle * f - r * car.config.width / 2.0,
                p - car.config.cg_to_rear_axle * f - r * car.config.width / 2.0,
            ];

            let dotp = car.dir().dot(car.velocity.sgn());

            if dotp < 0.9 {
                for i in 0..4 {
                    let p = wheel_p[i];
                    let prevp = self.prev_pos[j][i];
                    let d = p - prevp;
                    let r = Vec3::new(d.z, d.y, -d.x).sgn() * 0.2;
                    let u = Vec3::new(0.0, 0.01, 0.0);

                    let pts = vec![
                        u + prevp + r,
                        u + prevp - r,
                        u + p + r,
                        u + p + r,
                        u + prevp - r,
                        u + p - r,
                    ];

                    let r = (self.next_id as f32 / 100.0) % 1.0;
                    let tpt = vec![
                        Vec2::new(0.0, r + 0.01),
                        Vec2::new(1.0, r + 0.01),
                        Vec2::new(0.0, r),
                        Vec2::new(0.0, r),
                        Vec2::new(1.0, r + 0.01),
                        Vec2::new(1.0, r),
                    ];
                    let clr = vec![
                        Vec4::WHITE.with_w(
                            (car.speed().clamp(20.0, 40.0) - 20.0) / 20.0
                                * 2.0
                                * (1.0 - dotp.abs())
                        );
                        6
                    ];

                    self.pos_buf.update(&pts, self.next_id * 6);
                    self.clr_buf.update(&clr, self.next_id * 6);
                    self.tpt_buf.update(&tpt, self.next_id * 6);
                    self.next_id = (self.next_id + 1) % self.size;
                }
            }

            self.prev_pos[j] = wheel_p;
        }
    }
}

impl WheelMark {
    pub fn new(cars: Vec<Entity>, world: &mut StaticWorld) -> WheelMark {
        let size = 500;
        let vertices_per_item = 6;
        let pos_buf = Buffer::from_vec(&vec![Vec3::origin(); size * vertices_per_item]);
        let clr_buf = Buffer::from_vec(&vec![Vec4::WHITE; size * vertices_per_item]);
        let tpt_buf = Buffer::from_vec(&vec![Vec2::origin(); size * vertices_per_item]);

        let mut vao = VertexArray::new();
        vao.attrib_buffer(0, &pos_buf);
        vao.attrib_buffer(1, &clr_buf);
        vao.attrib_buffer(2, &tpt_buf);

        let tex = RgbaTexture::from_file("images/wheel_mark.png")
            .unwrap_or(RgbaTexture::new_color(1, 1, Vec4::BLACK));

        let mut render_sq = RenderSequence::new();
        render_sq.add_command(RenderCommand {
            vao,
            mode: DrawMode::Triangles,
            shader: DrawShaderSelector::Textured,
            uniforms: vec![Uniform::from("tex", &tex)],
            transparent: true,
            instances: 1,
            wireframe: false,
        });

        let e = world.new_entity_with_component(DrawComponent::from_render_seq(render_sq));
        world.add_component(
            e,
            BodyComponent {
                center: Vec3::new(0.0, 0.0, 0.0),
            },
        );

        WheelMark {
            cars,
            next_id: 0,
            pos_buf,
            clr_buf,
            tpt_buf,
            size,
            prev_pos: vec![[Vec3::origin(); 4]; size],
            _tex: tex,
        }
    }
}
