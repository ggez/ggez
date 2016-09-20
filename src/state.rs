use std::time::Duration;

use GameError;
use context::Context;

// I feel like this might be better named a Scene than a State...?
pub trait State {
    fn load(&mut self, ctx: &mut Context) -> Result<(), GameError>;
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> Result<(), GameError>;
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError>;
}
