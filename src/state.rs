use std::time::Duration;

use GameError;

pub trait State {
    fn load(&mut self) -> Result<(), GameError>;
    fn update(&mut self, dt: Duration) -> Result<(), GameError>;
    fn draw(&mut self) -> Result<(), GameError>;
}
