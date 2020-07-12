use glui::mecs::{DrawComponent, Entity, Message, StaticWorld, System, World};
use glui::tools::mesh::Mesh;
use glui::tools::{DrawShader, Mat4, Uniform, Vec3};

#[derive(Debug, Copy, Clone, Message)]
pub struct SetSunDir(pub Vec3);
#[derive(Debug, Copy, Clone, Message)]
pub struct SetRayleighK(pub f32);
#[derive(Debug, Copy, Clone, Message)]
pub struct SetMieK(pub f32);

pub struct Sky {
    draw_entity: Entity,
}

impl System for Sky {
    fn receive(&mut self, msg: &Box<dyn Message>, world: &mut StaticWorld) {
        if let Some(SetSunDir(d)) = msg.downcast_ref() {
            world.with_component_mut(self.draw_entity, |c: &mut DrawComponent| {
                c.render_seq.command_mut(0).uniforms[0] = Uniform::Vector3("L".to_owned(), *d);
            });
        }
        if let Some(SetMieK(k)) = msg.downcast_ref() {
            world.with_component_mut(self.draw_entity, |c: &mut DrawComponent| {
                c.render_seq.command_mut(0).uniforms[1] = Uniform::Float("mie_coef".to_owned(), *k);
            });
        }
        if let Some(SetRayleighK(k)) = msg.downcast_ref() {
            world.with_component_mut(self.draw_entity, |c: &mut DrawComponent| {
                c.render_seq.command_mut(0).uniforms[2] =
                    Uniform::Float("rayleigh_coef".to_owned(), *k);
            });
        }
    }
}

impl Sky {
    pub fn new(sun_dir: Vec3, world: &mut World) -> Sky {
        let mesh = Mesh::screen_quad();

        let render_seq = mesh.as_render_seq(
            DrawShader::from_files("shaders/sky.vert", "shaders/sky.frag")
                .unwrap()
                .into(),
            vec![
                Uniform::Vector3("L".to_owned(), sun_dir),
                Uniform::Float("mie_coef".to_owned(), 0.01),
                Uniform::Float("rayleigh_coef".to_owned(), 0.0045),
            ],
        );

        let quad_entity = world.entity();
        world.add_component(
            quad_entity,
            DrawComponent {
                render_seq,
                model_matrix: Mat4::identity(),
            },
        );
        Sky {
            draw_entity: quad_entity,
        }
    }
}
