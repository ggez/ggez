use std::collections::HashMap;

use std::path::{Path, PathBuf};
use sdl2::render::Texture;
use sdl2_image::LoadTexture;
use sdl2_mixer;
use sdl2_mixer::Music;
use sdl2_mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG,
                 AUDIO_S16LSB};
use sdl2_ttf::{self, Font, Sdl2TtfContext};

use GameError;

pub struct ResourceManager {
    textures: HashMap<String, Texture>,
    fonts: HashMap<(String, u16), Font>,
    font_type_faces: HashMap<String, PathBuf>,
    sounds: HashMap<String, Music>,
}

impl ResourceManager {
    pub fn new() -> Result<ResourceManager, GameError> {

        Ok(ResourceManager {
            textures: HashMap::new(),
            fonts: HashMap::new(),
            font_type_faces: HashMap::new(),
            sounds: HashMap::new(),
        })
    }

    pub fn load_sound(&mut self, name: &str, filename: &Path) -> Result<(), GameError> {
        let resource = sdl2_mixer::Music::from_file(filename);
        match resource {
            Ok(texture) => {
                self.sounds.insert(name.to_string(), texture);
                Ok(())
            }
            Err(msg) => Err(GameError::ResourceLoadError(msg)),
        }
    }

    pub fn get_sound(&self, name: &str) -> Option<&Music> {
        self.sounds.get(name)
    }
}

pub trait TextureManager {
    fn load_texture<T: LoadTexture>(&mut self,
                                    name: &str,
                                    filename: &Path,
                                    loader: &T)
                                    -> Result<(), GameError>;
    fn get_texture(&self, name: &str) -> Result<&Texture, GameError>;
}

impl TextureManager for ResourceManager {
    fn load_texture<T: LoadTexture>(&mut self,
                                    name: &str,
                                    filename: &Path,
                                    loader: &T)
                                    -> Result<(), GameError> {
        let resource = loader.load_texture(filename);
        match resource {
            Ok(texture) => {
                self.textures.insert(name.to_string(), texture);
                Ok(())
            }
            Err(msg) => Err(GameError::ResourceLoadError(msg)),
        }
    }

    fn get_texture(&self, name: &str) -> Result<&Texture, GameError> {
        self.textures
            .get(name)
            .ok_or(GameError::ResourceNotFound(String::from(name)))
    }
}

/*
#[test]
fn test_render_fonts() {
    let mut resource_manager = ResourceManager::new().expect("Init ResourceManager failed");

    resource_manager.load_font("Dejavu", "resources/DejaVuSerif.ttf").expect("File not found");
    assert_eq!(1, resource_manager.font_type_faces.len());

    resource_manager.get_font("Dejavu", 128).expect("Load font 128 failed");
    resource_manager.get_font("Dejavu", 54).expect("Load font 54 failed");
    assert_eq!(2, resource_manager.fonts.len());
}
*/
