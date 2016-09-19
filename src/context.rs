use sdl2::{self, Sdl};
use sdl2::video::Window;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2_ttf::{self, PartialRendering};

use rand::distributions::{IndependentSample, Range};
use rand::{self, Rng, Rand};

use resources::{ResourceManager, TextureManager, FontManager};
use GameError;

pub struct Context<'a> {
    pub sdl_context: Sdl,
    // TODO add mixer and ttf systems to enginestate
    pub resources: ResourceManager,
    pub renderer: Renderer<'a>,
}

impl<'a> Context<'a> {
    pub fn new(window_title: &str, screen_width: u32, screen_height: u32) -> Result<Context<'a>, GameError> {
        let sdl_context = try!(sdl2::init());
        let video = try!(sdl_context.video());
        let window = try!(video.window(window_title, screen_width, screen_height)
                          .position_centered()
                          .opengl()
                          .build());

        let mut renderer = try!(window.renderer()
                                 .accelerated()
                                .build());

        // Can creating a ResourceManager actually fail?
        // Only if it finds no resource files, perhaps...
        // But even then.
        let resources = ResourceManager::new().unwrap();

        Ok(Context {
            sdl_context: sdl_context,
            resources: resources,
            renderer: renderer,
        })
    }

    pub fn print(&mut self, text: &str, x: u32, y: u32) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0, 400);
        let target = Rect::new(between.ind_sample(&mut rng),
                               50,
                               between.ind_sample(&mut rng) as u32,
                               500);

        let mut font_texture = create_font_surface(text, "DejaVuSerif", 72, &mut self.resources)
                                   .unwrap()
                                   .blended(Color::rand(&mut rng))
                                   .map_err(|_| GameError::Lolwtf)
                                   .and_then(|s| {
                                       self.renderer
                                           .create_texture_from_surface(&s)
                                           .map_err(|_| GameError::Lolwtf)
                                   })
                                   .unwrap();

        self.renderer.copy(&mut font_texture, None, Some(target));
    }
}

fn create_font_surface<'a>(text: &'a str,
                           font_name: &str,
                           size: u16,
                           resource_manager: &'a mut ResourceManager)
                           -> Result<PartialRendering<'a>, GameError> {
    let mut font = try!(resource_manager.get_font(font_name, size));
    Ok(font.render(text))
}
