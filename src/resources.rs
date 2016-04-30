use std::collections::HashMap;
use sdl2::render::Texture;
use sdl2_image::LoadTexture;
use GameError;
use std::path::Path;

pub struct ResourceManager {
    images: HashMap<String, Texture>,
//    fonts: HashMap<String, >,
//    sounds: HashMap<String, >,
}

impl ResourceManager {
    pub fn new() -> ResourceManager {
        ResourceManager { images: HashMap::new() }
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
}
