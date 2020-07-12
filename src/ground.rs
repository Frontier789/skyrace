use glui::graphics::{DrawShaderSelector, RenderCommand, RenderSequence};
use glui::mecs::{DrawComponent, Entity, StaticWorld, System};
use glui::tools::{Buffer, DrawMode, Mat4, Uniform, Vec3, Vec4, VertexArray};

#[allow(dead_code)]
pub struct Ground {
    draw: Entity,
}

impl System for Ground {}

impl Ground {
    pub fn new(world: &mut StaticWorld) -> Ground {
        let mut pts = vec![];

        let n = 100;

        for x in -n..n + 1 {
            pts.push(Vec3::new(x as f32, 0.0, -n as f32));
            pts.push(Vec3::new(x as f32, 0.0, n as f32));
            pts.push(Vec3::new(-n as f32, 0.0, x as f32));
            pts.push(Vec3::new(n as f32, 0.0, x as f32));
        }

        let pbuf = Buffer::from_vec(&pts);
        let mut vao = VertexArray::new();
        vao.attrib_buffer(0, &pbuf);

        let mut render_seq = RenderSequence::new();

        render_seq.add_buffer(pbuf.into_base_type());

        render_seq.add_command(RenderCommand {
            vao,
            mode: DrawMode::Lines,
            shader: DrawShaderSelector::UniformColored,
            uniforms: vec![Uniform::from("color", Vec4::new(0.85, 0.55, 0.55, 1.0))],
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

        Ground { draw: e }
    }
}
