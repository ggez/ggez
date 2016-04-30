use std::time::Duration;

use GameError;
use context::Context;

pub trait State {
    fn load(&mut self, ctx: &mut Context) -> Result<(), GameError>;
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> Result<(), GameError>;
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError>;
}
