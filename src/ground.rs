use glui::graphics::{DrawShaderSelector, RenderCommand, RenderSequence};
use glui::mecs::{DrawComponent, Entity, StaticWorld, System};
use glui::tools::{Buffer, DrawMode, Mat4, RgbaTexture, Uniform, Vec2, Vec3, Vec4, VertexArray};

#[allow(dead_code)]
pub struct Ground {
    draw: Entity,
    tex: RgbaTexture,
}

impl System for Ground {}

impl Ground {
    pub fn new(world: &mut StaticWorld) -> Ground {
        let tex = RgbaTexture::from_file("images/track.bmp").unwrap();
        let s = tex.size() * 0.1;

        let pts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, s.y),
            Vec3::new(s.x, 0.0, s.y),
            Vec3::new(s.x, 0.0, 0.0),
        ];
        let clr = vec![Vec4::WHITE; 4];
        let tpt = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
        ];

        let pbuf = Buffer::from_vec(&pts);
        let cbuf = Buffer::from_vec(&clr);
        let tbuf = Buffer::from_vec(&tpt);
        let mut vao = VertexArray::new();
        vao.attrib_buffer(0, &pbuf);
        vao.attrib_buffer(1, &cbuf);
        vao.attrib_buffer(2, &tbuf);

        let mut render_seq = RenderSequence::new();

        render_seq.add_buffer(pbuf.into_base_type());
        render_seq.add_buffer(cbuf.into_base_type());
        render_seq.add_buffer(tbuf.into_base_type());

        render_seq.add_command(RenderCommand {
            vao,
            mode: DrawMode::TriangleFan,
            shader: DrawShaderSelector::Textured,
            uniforms: vec![
                Uniform::from("tex", &tex),
                Uniform::from("uv_matrix", Mat4::identity()),
            ],
            transparent: false,
            instances: 1,
        });

        let e = world.entity();
        world.add_component(
            e,
            DrawComponent {
                render_seq,
                model_matrix: Mat4::identity(),
            },
        );

        Ground { draw: e, tex }
    }
}
