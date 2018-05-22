//! Misc. input functions, currently just for handling gamepads.

use sdl2::GameControllerSubsystem;
use sdl2::Sdl;
pub use sdl2::controller::GameController;
use std::collections::HashMap;
use std::fmt;

use context::Context;
use error::GameResult;

/// To use gamepads (or joysticks) we need to "open" them
/// and keep them around, so this structure hangs on to
/// their state for us.
pub struct GamepadContext {
    /// Mapping of gamepad ID to controllers
    gamepads: HashMap<i32, GameController>,
    /// we need to keep the context around too
    #[allow(dead_code)]
    controller_ctx: GameControllerSubsystem,
}

impl fmt::Debug for GamepadContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<GamepadContext: {:p}>", self)
    }
}

impl GamepadContext {
    /// Create new GamepadContext
    pub fn new(sdl_context: &Sdl) -> GameResult<Self> {
        let controller_ctx = sdl_context.game_controller()?;
        let joy_count = controller_ctx.num_joysticks()?;
        let mut gamepads = HashMap::new();
        for i in 0..joy_count {
            if controller_ctx.is_game_controller(i) {
                let controller: GameController = controller_ctx.open(i)?;
                // gamepad events use this instance_id
                let id = controller.instance_id();
                gamepads.insert(id, controller);
            }
        }
        Ok(GamepadContext {
            gamepads,
            controller_ctx,
        })
    }
}

/// returns the `GameController` associated with an instance id.
/// The `instance_id` can be obtained from `GamepadEvents` in the `EventHandler`
pub fn get_gamepad(ctx: &Context, instance_id: i32) -> Option<&GameController> {
    ctx.gamepad_context.gamepads.get(&instance_id)
}
