extern crate notify;

use glui::graphics::{RenderCommand, RenderSequence};
use glui::mecs::{DrawComponent, Entity, Message, StaticWorld, System};
use glui::tools::mesh::parsurf_indices_triangulated;
use glui::tools::shader_error::ShaderLoadError;
use glui::tools::{
    Buffer, DrawMode, DrawShader, LinSpace, Mat4, RgbaTexture, Texture, Uniform, Vec2, Vec3,
    VertexArray,
};

use self::notify::DebouncedEvent;
use self::notify::DebouncedEvent::NoticeWrite;
use crate::utilities::watch;
use glui::tools::serde_tools::{SerdeError, SerdeJsonQuick};
use glui::tools::texture::TextureFiltering;
use glui::tools::texture_2d::ImageError;
use notify::RecommendedWatcher;
use std::sync::mpsc::Receiver;
use std::time::Duration;

#[derive(Clone, Debug, Message)]
pub struct ShowWireframe(pub bool);

pub struct Track {
    track_entity: Entity,
    shader: DrawShader,
    channel: (RecommendedWatcher, Receiver<DebouncedEvent>),
    textures: (RgbaTexture, RgbaTexture, RgbaTexture),
}

impl System for Track {
    fn receive(&mut self, msg: &Box<dyn Message>, world: &mut StaticWorld) {
        if let Some(ShowWireframe(show)) = msg.downcast_ref::<ShowWireframe>() {
            if let Some(c) = world.component_mut::<DrawComponent>(self.track_entity) {
                c.render_seq.command_mut(0).wireframe = *show;
            }
        }
    }

    fn update(&mut self, _delta_time: Duration, world: &mut StaticWorld) {
        let m = self.channel.1.try_recv();
        if let Ok(NoticeWrite(_)) = m {
            self.reload_shaders(world);
        }
        //
        // let cam = &world
        //     .component::<DataComponent<Camera>>(self.camera_entity)
        //     .unwrap()
        //     .data;
        // let cam_pos = cam.params.spatial.pos;
    }
}

#[derive(Debug)]
pub enum TrackError {
    ShaderLoadError(ShaderLoadError),
    NotifyError(notify::Error),
    ImageError(ImageError),
    SerdeError(SerdeError),
}

impl From<ShaderLoadError> for TrackError {
    fn from(e: ShaderLoadError) -> Self {
        TrackError::ShaderLoadError(e)
    }
}
impl From<notify::Error> for TrackError {
    fn from(e: notify::Error) -> Self {
        TrackError::NotifyError(e)
    }
}
impl From<ImageError> for TrackError {
    fn from(e: ImageError) -> Self {
        TrackError::ImageError(e)
    }
}
impl From<SerdeError> for TrackError {
    fn from(e: SerdeError) -> Self {
        TrackError::SerdeError(e)
    }
}

fn load_tex(path: &str) -> Result<RgbaTexture, ImageError> {
    let tex = RgbaTexture::from_file(path)?;
    tex.generate_mipmaps();
    tex.set_filtering(TextureFiltering::Linear);
    Ok(tex)
}
impl Track {
    pub fn new(world: &mut StaticWorld) -> Result<Track, TrackError> {
        let diffuse = load_tex("images/Road007_2K_Color_low.jpg")?;
        let normal = load_tex("images/Road007_2K_Normal_low.jpg")?;
        let roughness = load_tex("images/Road007_2K_Roughness_low.jpg")?;
        let mut track = Track {
            track_entity: world.entity(),
            shader: DrawShader::from_files("shaders/track.vert", "shaders/track.frag")?,
            channel: watch(vec!["shaders/track.frag"])?,
            textures: (diffuse, normal, roughness),
        };
        track.generate(world)?;
        Ok(track)
    }
    fn fix_curves(mut pts: Vec<Vec2>) -> Vec<Vec2> {
        let n = pts.len();
        pts[n - 1] = pts[0];
        let l = (pts[1] - pts[0]).length();
        let d = (pts[n - 1] - pts[n - 2]).sgn();
        pts[1] = pts[0] + d * l;

        pts
    }
    pub fn track_curve_control_points() -> Result<Vec<Vec2>, SerdeError> {
        let data_f32 = Vec::<f32>::load_json("race_track.json")?;
        let mut data = Vec::with_capacity(data_f32.len() / 2);
        for i in 0..data_f32.len() / 2 {
            data.push(Vec2::new(data_f32[i * 2 + 0], data_f32[i * 2 + 1]));
        }
        Ok(Self::fix_curves(data))
    }
    pub fn track_width() -> f32 {
        3.6 * 8.0
    }
    fn generate(&mut self, world: &mut StaticWorld) -> Result<(), SerdeError> {
        let mut pts = vec![];
        let mut tpts = vec![];
        let mut tang = vec![];
        let track_width = Self::track_width();
        let data = Self::track_curve_control_points()?;
        let mut i = 0;
        let mut last_p = Vec2::zero();
        let mut curve_len = 0.0;
        let mut sample_count = 0;
        while i + 4 <= data.len() {
            let p0 = data[i + 0];
            let p1 = data[i + 1];
            let p2 = data[i + 2];
            let p3 = data[i + 3];

            for t in (0.0..1.0).linspace(20) {
                let (p, _v, n) = Vec2::eval_bezier4(p0, p1, p2, p3, t);

                if i > 0 || t > 0.0 {
                    curve_len += (last_p - p).length();
                }
                last_p = p;

                pts.push(Vec3::from_vec2(p - n * track_width / 2.0, 0.0).xzy());
                pts.push(Vec3::from_vec2(p, 0.0).xzy());
                pts.push(Vec3::from_vec2(p + n * track_width / 2.0, 0.0).xzy());
                tpts.push(Vec2::new(0.0, curve_len / track_width));
                tpts.push(Vec2::new(0.5, curve_len / track_width));
                tpts.push(Vec2::new(1.0, curve_len / track_width));
                tang.push(Vec3::from_vec2(n, 0.0).xzy());
                tang.push(Vec3::from_vec2(n, 0.0).xzy());
                tang.push(Vec3::from_vec2(n, 0.0).xzy());
                sample_count += 1;
            }

            i += 3;
        }

        let inds = parsurf_indices_triangulated(3, sample_count);

        let pbuf = Buffer::from_vec(&pts);
        let tbuf = Buffer::from_vec(&tpts);
        let gbuf = Buffer::from_vec(&tang);
        let ibuf = Buffer::from_vec(&inds);
        let mut vao = VertexArray::new();
        vao.attrib_buffer(0, &pbuf);
        vao.attrib_buffer(2, &tbuf);
        vao.attrib_buffer(3, &gbuf);

        vao.set_indices_buffer(&ibuf);

        let mut render_seq = RenderSequence::new();

        render_seq.add_buffer(pbuf.into_base_type());
        render_seq.add_buffer(tbuf.into_base_type());
        render_seq.add_buffer(gbuf.into_base_type());
        render_seq.add_index_buffer(ibuf);

        let cmd = RenderCommand::new_uniforms(
            vao,
            DrawMode::Triangles,
            self.shader.clone().into(),
            vec![
                Uniform::from("color_tex", &self.textures.0),
                Uniform::from("normal_tex", &self.textures.1),
                Uniform::from("rougness_tex", &self.textures.2),
                Uniform::from("light_direction", Vec3::new(1.0, 1.0, 1.0).sgn()),
            ],
        );
        render_seq.add_command(cmd);

        let comp = DrawComponent {
            render_seq,
            model_matrix: Mat4::identity(),
        };

        world.add_component(self.track_entity, comp);

        Ok(())
    }
    fn reload_shaders(&mut self, world: &mut StaticWorld) {
        match DrawShader::from_files("shaders/track.vert", "shaders/track.frag") {
            Ok(shader) => {
                self.shader = shader;
                if let Some(comp) = world.component_mut::<DrawComponent>(self.track_entity) {
                    comp.render_seq.command_mut(0).shader = self.shader.clone().into();
                }
            }
            Err(e) => {
                println!("Failed to reload shaders: {:?}", e);
            }
        }
    }
}
