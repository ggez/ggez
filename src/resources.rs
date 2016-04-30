use std::path::Path;
use std::collections::HashMap;

use sdl2::render::Texture;
use sdl2_image::LoadTexture;
use sdl2_mixer;
use sdl2_mixer::Music;
use sdl2_mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG,
                 AUDIO_S16LSB};

use GameError;

pub struct ResourceManager {
    images: HashMap<String, Texture>,
//    fonts: HashMap<String, >,
    sounds: HashMap<String, Music>
}

impl ResourceManager {
    pub fn new() -> ResourceManager {
        ResourceManager
        {
            images: HashMap::new(),
            sounds: HashMap::new()
        }
    }

    pub fn load_texture<T: LoadTexture>(&mut self, name: &str, filename: &Path, loader: &T) -> Result<(), GameError> {
        let resource = loader.load_texture(filename);
        match resource {
            Ok(texture) => {
                self.images.insert(name.to_string(), texture);
                Ok(())
            }
            Err(msg) => Err(GameError::ResourceLoadError(msg))
        }
    }

    pub fn get_texture(&self, name: &str) -> Option<&Texture> {
        self.images.get(name)
    }

    pub fn load_sound(&mut self, name: &str, filename: &Path) -> Result<(), GameError> {
        let resource = sdl2_mixer::Music::from_file(filename);
        match resource {
            Ok(texture) => {
                self.sounds.insert(name.to_string(), texture);
                Ok(())
            }
            Err(msg) => Err(GameError::ResourceLoadError(msg))
        }
    }

    pub fn get_sound(&self, name: &str) -> Option<&Music> {
        self.sounds.get(name)
    }
}

/*    fn hook_finished() {
        println!("play ends! from rust cb");
    }

    sdl2_mixer::Music::hook_finished(hook_finished);

    println!("music => {:?}", music);
    println!("music type => {:?}", music.get_type());
    println!("music volume => {:?}", sdl2_mixer::Music::get_volume());
    println!("play => {:?}", music.play(1));

    timer.delay(10000);

    println!("fading out ... {:?}", sdl2_mixer::Music::fade_out(4000));

    timer.delay(5000);

    println!("fading in from pos ... {:?}",
             music.fade_in_from_pos(1, 10000, 100.0));

    timer.delay(5000);
    sdl2_mixer::Music::halt();
    timer.delay(1000);

}
*/
