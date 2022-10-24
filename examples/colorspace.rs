//! An example demonstrating sRGB color spaces.
//!
//! sRGB is one of those things that's a bit subtle, a bit obscure,
//! and easy to get wrong, but worth knowing because it crops up
//! from time to time and makes things mysteriously "look wrong" in
//! some cases.  It also is sort of overloaded to do two things at
//! once.  The first is what you will find [Wikipedia talking about](https://en.wikipedia.org/wiki/Srgb),
//! which is "how do you prove that 'green' on my monitor is the same
//! as 'green' on your monitor?".  That part we can safely ignore,
//! since it's a job for your monitor manufacturer.
//!
//! The other part is [gamma
//! correction](https://en.wikipedia.org/wiki/Gamma_correction) which
//! deals with the fact that the response of the human visual system
//! is non-linear, or in non-science talk, if you make a pixel twice
//! as bright, it doesn't *look* twice as bright.  To make something
//! *look* twice as bright, you have to make it about 2^2.2 times
//! brighter.  The exact math for how to fiddle the numbers to make
//! things Look Nice is defined as part of the sRGB standard, so often
//! a color scheme that complies with this gamma correction process is
//! just called "sRGB".
//!
//! In a perfect world, we don't ever have to worry about this.
//! Images are generally all stored in the sRGB color space (or
//! something similar to it), monitors all display in the sRGB color
//! space, and our graphics drivers know how to do whatever is
//! necessary to make the two line up.  The problem comes because we
//! are programmers, and have to be able to poke things instead of
//! just using pre-loaded assets.  So the question is: if you do
//! `Color::new(0.5, 0.0, 0.0, 1.0)` and `Color::new(1.0, 0.0, 0.0, 1.0)`,
//! will the second color LOOK twice as bright as the first one?
//!
//! So we have to know what color space we are talking about when we
//! say `Color`!  Are we talking about linear, "real" color where your
//! pixel puts out twice as many photons for the second color as the
//! first?  Or are we talking about sRGB color, where the pixel
//! actually LOOKS twice as bright?  And if we want to, say, write a
//! shader that does math to these colors, AND to colors that come
//! from images that use the sRGB color space, how do we make sure
//! everything matches?  To make it even worse, the sRGB conversion
//! done by graphics drivers is toggle-able, and can be set on a
//! per-render-target or per-texture basis, so it's possible for
//! things to get REAL mixed up in subtle ways.
//!
//! The Right Answer, as far as I know, is this: All colors that a
//! human specifies or touches are sRGB-encoded, so a number that is
//! twice as big then .  We do our color math in shaders, and (if we
//! set our render target to be an sRGB texture, which ggez always
//! does) the graphics driver will turn the linear colors we specify
//! into sRGB colors.  So if we do `vec4 x = vec4(0.25, 0.0, 0.0, 1.0);`,
//! assigning one pixel `x` and another `x * 2` will make the
//! second one *look* twice as bright as the first.
//!
//! BUT, this process also has to be done on INPUT as well; if we pass
//! the shader a value taken from an image file, that image file is in
//! sRGB color.  The graphics driver must THEN convert the sRGB color into
//! a linear color when passing it to the shader, so that if we get the
//! color and call it `x`, assigning one output pixel `x` and another
//! `x * 2` again makes the second one LOOK twice as bright as the first.
//! Then it converts the value back on the way out.
//!
//! ggez should handle all of this for you.  `graphics::Color` is
//! explicitly a sRGB-corrected color, all textures including the
//! final render target are sRGB-enabled, and when you provide
//! a linear color to something like `graphics::Mesh` it turns it
//! into sRGB for you to match everything else.  The purpose of this
//! example is to show that this actually *works* correctly!

use ggez::event;
use ggez::glam::*;
use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};

/// This is a nice aqua test color that will look a lot brighter
/// than it should if we mess something up.
/// See https://github.com/ggez/ggez/issues/209 for examples.
const AQUA: graphics::Color = graphics::Color::new(0.0078, 0.7647, 0.6039, 1.0);

struct MainState {
    demo_mesh: graphics::Mesh,
    square_mesh: graphics::Mesh,
    demo_image: graphics::Image,
    demo_instances: graphics::InstanceArray,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let demo_mesh = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            100.0,
            2.0,
            AQUA,
        )?;
        let square_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, 400.0, 400.0),
            Color::WHITE,
        )?;
        let demo_image = graphics::Image::from_solid(ctx, 200, AQUA);

        let mut demo_instances = graphics::InstanceArray::new(ctx, demo_image.clone());
        demo_instances.push(
            DrawParam::default()
                .dest(Vec2::new(250.0, 350.0))
                .scale(Vec2::new(0.25, 0.25)),
        );
        demo_instances.push(
            DrawParam::default()
                .dest(Vec2::new(250.0, 425.0))
                .scale(Vec2::new(0.1, 0.1)),
        );

        let s = MainState {
            demo_mesh,
            square_mesh,
            demo_image,
            demo_instances,
        };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, AQUA);

        // Draw a white square so we can see things
        canvas.draw(
            &self.square_mesh,
            DrawParam::default().dest(Vec2::new(200.0, 100.0)),
        );

        // Draw things partially over the white square so we can see
        // where they are; they SHOULD be the same color as the
        // background.

        // mesh
        canvas.draw(
            &self.demo_mesh,
            DrawParam::default().dest(Vec2::new(150.0, 200.0)),
        );

        // image
        canvas.draw(
            &self.demo_image,
            DrawParam::default().dest(Vec2::new(450.0, 200.0)),
        );

        // text
        canvas.draw(
            graphics::Text::new("-").set_scale(300.),
            graphics::DrawParam::from([150., 135.]).color(AQUA),
        );

        // instancearray
        canvas.draw(
            &self.demo_instances,
            DrawParam::default().dest(Vec2::new(0.0, 0.0)),
        );

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    use std::env;
    use std::path;
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("colorspace", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
