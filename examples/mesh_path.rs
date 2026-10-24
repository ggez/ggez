//! Draws curved shapes by building a `Path` and tessellating it with `MeshBuilder::path`.

use ggez::{
    event,
    glam::*,
    graphics::{
        self,
        path::{
            builder::SvgPathBuilder,
            math::{point, vector, Angle, Point},
            ArcFlags, Path,
        },
        Color, DrawMode,
    },
    Context, GameResult,
};

struct MainState {
    mesh: graphics::Mesh,
}

fn p(v: Vec2) -> Point {
    point(v.x, v.y)
}

/// A heart made of two cubic Bézier curves.
fn heart(c: Vec2) -> Path {
    let mut path = Path::builder();
    path.begin(p(c + vec2(0., 60.)));
    path.cubic_bezier_to(
        p(c + vec2(-110., -10.)),
        p(c + vec2(-50., -90.)),
        p(c + vec2(0., -40.)),
    );
    path.cubic_bezier_to(
        p(c + vec2(50., -90.)),
        p(c + vec2(110., -10.)),
        p(c + vec2(0., 60.)),
    );
    path.end(true);
    path.build()
}

/// An open wave made of quadratic Bézier curves.
fn wave(start: Vec2) -> Path {
    let mut path = Path::builder();
    path.begin(p(start));
    for i in 0..6 {
        let x = start.x + i as f32 * 80.;
        path.quadratic_bezier_to(p(vec2(x + 40., start.y - 60.)), p(vec2(x + 80., start.y)));
    }
    path.end(false);
    path.build()
}

/// Line from center, SVG-style arc, then back to the center to form pacman
fn pacman(c: Vec2) -> Path {
    let mut path = Path::builder().with_svg();
    path.move_to(p(c));
    path.line_to(p(c + vec2(50., -30.)));
    path.arc_to(
        vector(58.3, 58.3),
        Angle::zero(),
        ArcFlags {
            large_arc: true,
            sweep: false,
        },
        p(c + vec2(50., 30.)),
    );
    path.close();
    path.build()
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut mb = graphics::MeshBuilder::new();
        mb.path(DrawMode::fill(), &heart(vec2(150., 180.)), Color::RED)?
            .path(DrawMode::stroke(6.), &heart(vec2(400., 180.)), Color::WHITE)?
            .path(DrawMode::fill(), &pacman(vec2(630., 180.)), Color::YELLOW)?
            .path(DrawMode::stroke(4.), &wave(vec2(160., 420.)), Color::CYAN)?;

        Ok(MainState {
            mesh: graphics::Mesh::from_data(ctx, mb.build()),
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));
        canvas.draw(&self.mesh, graphics::DrawParam::new());
        canvas.finish(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("mesh_path", "ggez");
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
