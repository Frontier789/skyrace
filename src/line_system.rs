use glui::graphics::{DrawShaderSelector, RenderCommand, RenderSequence};
use glui::mecs::{DrawComponent, Entity, Message};
use glui::mecs::{StaticWorld, System};
use glui::tools::{Buffer, DrawMode, Vec3, Vec4, VertexArray};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct LineDesc {
    pub color: Vec4,
    pub a: Vec3,
    pub b: Vec3,
}

#[derive(Debug, Copy, Clone, Message)]
pub struct LinesUpdate {}

#[derive(Debug, Clone, Message)]
pub struct SetLine(pub String, pub LineDesc);

pub struct LineSystem {
    lines: HashMap<String, LineDesc>,
    pts: Buffer<Vec3>,
    col: Buffer<Vec4>,
    draw: Entity,
}

impl LineSystem {
    pub fn new(world: &mut StaticWorld) -> LineSystem {
        let vao = VertexArray::new();
        let mut render_seq = RenderSequence::new();

        let pts = Buffer::new();
        let col = Buffer::new();

        render_seq.add_command(RenderCommand {
            vao,
            mode: DrawMode::Lines,
            shader: DrawShaderSelector::Colored,
            uniforms: vec![],
            transparent: false,
            instances: 1,
        });

        let e = world.entity();
        world.add_component(e, DrawComponent::from_render_seq(render_seq));

        LineSystem {
            lines: Default::default(),
            pts,
            col,
            draw: e,
        }
    }
}

impl System for LineSystem {
    fn receive(&mut self, msg: &Box<dyn Message>, world: &mut StaticWorld) {
        if let Some(line) = msg.downcast_ref::<SetLine>() {
            self.lines.insert(line.0.clone(), line.1);
        }
        if let Some(_) = msg.downcast_ref::<LinesUpdate>() {
            if self.lines.is_empty() {
                return;
            }

            let pts_vec = self
                .lines
                .iter()
                .map(|(_name, line)| vec![line.a, line.b])
                .flatten()
                .collect::<Vec<Vec3>>();
            let col_vec = self
                .lines
                .iter()
                .map(|(_name, line)| vec![line.color, line.color])
                .flatten()
                .collect::<Vec<Vec4>>();

            self.pts.set_data(&pts_vec);
            self.col.set_data(&col_vec);

            let draw_comp = world.component_mut::<DrawComponent>(self.draw).unwrap();

            let vao = &mut draw_comp.render_seq.command_mut(0).vao;

            vao.attrib_buffer(0, &self.pts);
            vao.attrib_buffer(1, &self.col);
            vao.set_indices_range(0..self.lines.len() * 2);
        }
    }
}
