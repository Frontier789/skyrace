use glui::graphics::{DrawShaderSelector, RenderCommand, RenderSequence};
use glui::mecs::{DrawComponent, Entity, StaticWorld, System};
use glui::tools::{
    parsurf, parsurf_indices, Buffer, DrawMode, DrawShader, FloatTexture, LinSpace, Mat4,
    RgbaTexture, Uniform, Vec2, Vec3, VertexArray,
};

use crate::utilities::watch;
use noise::{NoiseFn, OpenSimplex};
use notify::DebouncedEvent::NoticeWrite;
use notify::{DebouncedEvent, RecommendedWatcher};
use std::sync::mpsc::Receiver;
use std::time::Duration;

struct HmapSampler {
    gen: OpenSimplex,
}

impl HmapSampler {
    pub fn new() -> HmapSampler {
        HmapSampler {
            gen: OpenSimplex::new(42),
        }
    }
    const SCALE: f32 = 10.0;

    pub fn get(&self, p: Vec2) -> f32 {
        let s = Self::SCALE;
        let mut h = 0.0;
        h += self.gen.get([(p.x * s) as f64, (p.y * s) as f64]) as f32;
        h += self
            .gen
            .get([(p.x * s * 4.0) as f64, (p.y * s * 4.0) as f64]) as f32
            / 16.0;
        if h < 0.0 {
            h = -(1.0 - h).ln()
        }
        h * 60.0
    }
}

#[allow(dead_code)]
pub struct Terrain {
    draw: Entity,
    height_tex: FloatTexture,
    norm_tex: RgbaTexture,
    tang_tex: RgbaTexture,
    sand: RgbaTexture,
    sand_norm: RgbaTexture,
    channel: (RecommendedWatcher, Receiver<DebouncedEvent>),
    shader: DrawShader,
    hmap: HmapSampler,
}

impl System for Terrain {
    fn update(&mut self, _delta_time: Duration, world: &mut StaticWorld) {
        let m = self.channel.1.try_recv();
        if let Ok(NoticeWrite(_)) = m {
            self.reload_shaders(world);
        }
    }
}

impl Terrain {
    pub fn new(world: &mut StaticWorld) -> Terrain {
        let hmap = HmapSampler::new();
        let (norm_tex, height_tex, tang_tex, w, h) = Self::gen_texes(&hmap);
        let sand = RgbaTexture::from_file("images/sand.jpg").unwrap_or(RgbaTexture::unit());
        let sand_norm =
            RgbaTexture::from_file("images/sand_norm.jpg").unwrap_or(RgbaTexture::unit());

        let pts = parsurf(|x, y| Vec2::new(x, y), w, h);
        let inds = parsurf_indices(w, h);

        let pbuf = Buffer::from_vec(&pts);
        let ibuf = Buffer::from_vec(&inds);
        let mut vao = VertexArray::new();
        vao.attrib_buffer(0, &pbuf);
        vao.set_indices_buffer(&ibuf);

        let mut render_seq = RenderSequence::new();

        render_seq.add_buffer(pbuf.into_base_type());
        render_seq.add_index_buffer(ibuf);

        let shader = DrawShader::from_files("shaders/terrain.vert", "shaders/terrain.frag")
            .expect("Failed to load Terrain shader!");
        render_seq.add_command(RenderCommand::new_uniforms(
            vao,
            DrawMode::TriangleStrip,
            DrawShaderSelector::Custom(shader.clone()),
            vec![
                Uniform::from("height_tex", &height_tex),
                Uniform::from("norm_tex", &norm_tex),
                Uniform::from("tang_tex", &tang_tex),
                Uniform::from("sand", &sand),
                Uniform::from("sand_norm", &sand_norm),
                Uniform::from("light_direction", Vec3::new(1.0, 0.3, 1.0)),
            ],
        ));

        let e = world.entity();
        world.add_component(
            e,
            DrawComponent {
                render_seq,
                model_matrix: Mat4::identity(),
            },
        );

        Terrain {
            draw: e,
            height_tex,
            norm_tex,
            shader,
            channel: watch(vec!["shaders/terrain.vert", "shaders/terrain.frag"]).unwrap(),
            sand,
            tang_tex,
            sand_norm,
            hmap,
        }
    }

    pub fn height_tex(&self) -> &FloatTexture {
        &self.height_tex
    }

    fn reload_shaders(&mut self, world: &mut StaticWorld) {
        match DrawShader::from_files("shaders/terrain.vert", "shaders/terrain.frag") {
            Ok(shader) => {
                self.shader = shader;
                if let Some(comp) = world.component_mut::<DrawComponent>(self.draw) {
                    comp.render_seq.command_mut(0).shader = self.shader.clone().into();
                }
            }
            Err(e) => {
                println!("Failed to reload terrain shaders: {:?}", e);
            }
        }
    }

    fn gen_texes(hmap: &HmapSampler) -> (RgbaTexture, FloatTexture, RgbaTexture, usize, usize) {
        let level = RgbaTexture::load_rgba_image("images/level.png").unwrap();
        let width = level.width() as usize;
        let height = level.height() as usize;
        let mut data = Vec::with_capacity(width * height);
        for x in (0.0..1.0f32).linspace(width) {
            for y in (0.0..1.0f32).linspace(height) {
                data.push(hmap.get(Vec2::new(x, y)));
            }
        }

        for i in 0..width {
            for j in 0..height {
                let f = level
                    .get_pixel(height as u32 - 1 - j as u32, width as u32 - 1 - i as u32)
                    .0[0] as f32
                    / 255.0;
                data[i * height + j] *= f;
            }
        }

        let mut nrm_pxs = Vec::with_capacity(width * height);
        let mut tan_pxs = Vec::with_capacity(width * height);
        for i in 0..width - 1 {
            for j in 0..height - 1 {
                let h = data[i * height + j];
                let h1 = data[(i + 1) * height + j];
                let h2 = data[i * height + (j + 1)];

                let n = Vec3::new((h - h1) * 10.0, 1.0, (h - h2) * 10.0).sgn();
                nrm_pxs.push(n);

                let t = Vec3::new(1.0, -(h - h1) * 10.0, 0.0).sgn();
                tan_pxs.push(t);
            }
            nrm_pxs.push(Vec3::zero());
            tan_pxs.push(Vec3::zero());
        }
        for _ in 0..height {
            nrm_pxs.push(Vec3::zero());
            tan_pxs.push(Vec3::zero());
        }
        (
            RgbaTexture::from_vec_v3_rescale(width, height, &nrm_pxs),
            FloatTexture::from_vec(width, height, &data),
            RgbaTexture::from_vec_v3_rescale(width, height, &tan_pxs),
            width,
            height,
        )
    }
}
