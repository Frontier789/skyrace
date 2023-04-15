use glui::mecs::{DrawComponent, Entity, StaticWorld, System};
use glui::tools::mesh::Mesh;
use glui::tools::{DrawShader, FloatTexture, Mat4, RgbaTexture, Uniform, Vec2, Vec3, Vec4};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::f32::consts::PI;

#[allow(dead_code)]
pub struct Cacti {
    draws: Vec<Entity>,
    tex: RgbaTexture,
    shader: DrawShader,
}

impl System for Cacti {}

impl Cacti {
    pub fn new(world: &mut StaticWorld, height_tex: &FloatTexture) -> Cacti {
        let cactus01 = match Mesh::load_obj("models/cactus01.obj") {
            Ok(mesh) => mesh,
            Err(_) => Mesh::unit_cylinder(9),
        }
        .upload_to_gpu();
        let cactus02 = match Mesh::load_obj("models/cactus02.obj") {
            Ok(mesh) => mesh,
            Err(_) => Mesh::unit_cylinder(9),
        }
        .upload_to_gpu();
        let cactus03 = match Mesh::load_obj("models/cactus03.obj") {
            Ok(mesh) => mesh,
            Err(_) => Mesh::unit_cylinder(9),
        }
        .upload_to_gpu();

        let tex = RgbaTexture::from_file("models/cactus_atlas.png")
            .unwrap_or(RgbaTexture::new_color(1, 1, Vec4::GREEN));

        let seed: [u8; 16] = [
            123, 111, 53, 63, 133, 101, 54, 43, 76, 4, 16, 0, 77, 91, 1, 42,
        ];
        let mut rng: XorShiftRng = SeedableRng::from_seed(seed);
        let mut unit_rand = || rng.gen_range(0.0..=1.0);

        let n = 500;

        let shader = DrawShader::from_files("shaders/cactus.vert", "shaders/cactus.frag")
            .expect("Failed to load cactus shaders!");

        let mut draws = Vec::with_capacity(n);
        for _ in 0..n {
            let p = Vec2::new(unit_rand(), unit_rand());
            let i = unit_rand();
            let cactus = if i < 0.3 {
                &cactus01
            } else if i < 0.7 {
                &cactus02
            } else {
                &cactus03
            };
            let rs = cactus.non_owning_render_seq(
                shader.clone().into(),
                vec![
                    Uniform::from("diffuse_tex", &tex),
                    Uniform::from("light_direction", Vec3::new(1.0, 0.0, 0.0)),
                    Uniform::from("offset", p),
                    Uniform::from("hmap", height_tex),
                ],
            );
            let mut comp = DrawComponent::from_render_seq(rs);
            comp.model_matrix =
                Mat4::scale(5.0 + unit_rand() * 5.0) * Mat4::rotate_y(unit_rand() * PI * 2.0);
            draws.push(world.new_entity_with_component(comp));
        }

        Cacti { draws, tex, shader }
    }
}
