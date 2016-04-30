use std::time::Duration;

use GameError;
use game::Context;

pub trait State {
    fn init(&mut self, ctx: &mut Context) -> Result<(), GameError>;
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> Result<(), GameError>;
    fn draw(&mut self) -> Result<(), GameError>;
}
