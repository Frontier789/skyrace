use crate::spline::Spline;
use glui::gui::{CallbackExecutor, EventResponse, GuiCallback, WidgetAdder};
use glui::gui::{DrawBuilder, Widget, WidgetConstraints, WidgetParser, WidgetSize};
use glui::impl_widget_building_for;
use glui::tools::{LinSpace, Rect, Vec2, Vec2px, Vec3, Vec4};
use std::ops::{Neg, Shl};

#[derive(Default, Clone)]
pub struct SplinePrivate {
    real_size: Vec2px,
    hover: Option<usize>,
    grab: Option<usize>,
    grab_offset: Vec3,
}

#[derive(Clone, Default)]
pub struct SplineEditor {
    pub color: Vec4,
    pub size: WidgetSize,
    pub private: SplinePrivate,
    pub points: Vec<Vec3>,
    pub callback: GuiCallback<SplineEditor>,
}

impl_widget_building_for!(SplineEditor);
impl Widget for SplineEditor {
    fn constraint(&mut self, self_constraint: WidgetConstraints) {
        self.private.real_size = self.size.to_units(self_constraint.max_size);
    }
    fn on_press(
        &mut self,
        local_cursor_pos: Vec2px,
        executor: &mut CallbackExecutor,
    ) -> EventResponse {
        let p = Vec3::new(local_cursor_pos.x, 0.0, local_cursor_pos.y);

        let (j, d, o) = self.closest_to(p);

        if d < 15.0 {
            self.set_point(j, p + o);
            executor.execute(&self.callback, &self);
            self.private.grab = Some(j);
            self.private.grab_offset = o;
            EventResponse::HandledRedraw
        } else {
            EventResponse::Handled
        }
    }
    fn on_release(&mut self, _executor: &mut CallbackExecutor) -> EventResponse {
        self.private.grab = None;
        self.private.hover = None;
        EventResponse::Handled
    }

    fn on_cursor_move(
        &mut self,
        local_cursor_pos: Vec2px,
        executor: &mut CallbackExecutor,
    ) -> EventResponse {
        let p = Vec3::new(local_cursor_pos.x, 0.0, local_cursor_pos.y);

        if let Some(id) = self.private.grab {
            self.set_point(id, p + self.private.grab_offset);
            executor.execute(&self.callback, &self);
            EventResponse::HandledRedraw
        } else {
            let (j, d, _) = self.closest_to(p);
            let hov = if d < 15.0 { Some(j) } else { None };

            let resp = if hov != self.private.hover {
                EventResponse::HandledRedraw
            } else {
                EventResponse::Handled
            };

            self.private.hover = hov;
            resp
        }
    }
    fn on_draw_build(&self, builder: &mut DrawBuilder) {
        let spline = Spline::fit_cubic((0.0..1.0).linspace(self.points.len()), self.points.clone());
        let pts = spline
            .quantize(100)
            .iter()
            .map(|p| Vec2px::from_pixels(p.xz(), 1.0))
            .collect();

        // println!("{:?}", pts);

        builder.add_clr_rect(
            Rect::from_pos_size(Vec2::origin(), self.size().to_pixels(1.0)),
            Vec4::WHITE.with_w(0.2),
        );
        builder.add_line_strip(pts, self.color);
        for i in 0..self.points.len() {
            let p = self.points[i];
            let c = if self.private.grab == Some(i) {
                Vec4::grey(0.4)
            } else if self.private.grab == None && self.private.hover == Some(i) {
                Vec4::grey(0.6)
            } else {
                Vec4::WHITE
            };

            builder.add_tex(Vec2px::from_pixels(p.xz(), 1.0), "images/dot", c, 1.0 / 6.0);
        }
    }
    fn size(&self) -> Vec2px {
        self.private.real_size
    }
}

impl SplineEditor {
    fn set_point(&mut self, id: usize, mut pos: Vec3) {
        let s = self.size();
        if pos.x < 0.0 {
            pos.x = 0.0;
        }
        if pos.z < 0.0 {
            pos.z = 0.0;
        }
        if pos.x > s.x {
            pos.x = s.x;
        }
        if pos.z > s.y {
            pos.z = s.y;
        }
        self.points[id] = pos;
    }

    fn closest_to(&self, p: Vec3) -> (usize, f32, Vec3) {
        let mut d = (self.points[0] - p).length();
        let mut o = self.points[0] - p;
        let mut j = 0;
        for i in 1..self.points.len() {
            let dc = (self.points[i] - p).length();
            if dc < d {
                j = i;
                d = dc;
                o = self.points[i] - p
            }
        }

        (j, d, o)
    }
}
