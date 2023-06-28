// This is ugly and hacky but greatly improves ergonomics.

/// Used to represent types that can provide a certain context type.
///
/// If you don't know what this is, you most likely want to pass `ctx`.
///
/// This trait is basically syntactical sugar, saving you from having
/// to split contexts when you don't need to and also shortening calls like
/// ```rust
/// # use ggez::GameResult;
/// # fn t(ctx: &mut ggez::Context, canvas: ggez::graphics::Canvas) -> GameResult {
/// canvas.finish(&mut ctx.gfx)?;
/// # Ok(())
/// # }
/// ```
/// into just
/// ```rust
/// # use ggez::GameResult;
/// # fn t(ctx: &mut ggez::Context, canvas: ggez::graphics::Canvas) -> GameResult {
/// canvas.finish(ctx)?;
/// # Ok(())
/// # }
/// ```
pub trait Has<T> {
    /// Method to retrieve the type.
    fn retrieve(&self) -> &T;
}

impl<T> Has<T> for T {
    #[inline]
    fn retrieve(&self) -> &T {
        self
    }
}

/// Used to represent types that can provide a certain context type in a mutable form.
/// See also [`Has<T>`].
///
/// If you don't know what this is, you most likely want to pass `ctx`.
pub trait HasMut<T> {
    /// Method to retrieve the type as mutable.
    fn retrieve_mut(&mut self) -> &mut T;
}

impl<T> HasMut<T> for T {
    #[inline]
    fn retrieve_mut(&mut self) -> &mut T {
        self
    }
}

pub mod prelude {
    pub use crate::{Has, HasMut};
}
