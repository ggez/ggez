```rust,template-draw
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;
use rand::*;

enum Shape {
    Circle(mint::Point2<f32>, f32),
    Rectangle(graphics::Rect),
}

struct State {
    shapes: Vec<Shape>,
}

impl State {
    fn new(_ctx: &mut Context) -> GameResult<Self> {
        let s = State { shapes: Vec::new() };
        Ok(s)
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    {{example}}
}

pub fn main() -> GameResult {
    Ok(())
}
```

```rust,template-expression
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;
use rand::*;
use oorandom::Rand32;

enum Shape {
    Circle(mint::Point2<f32>, f32),
    Rectangle(graphics::Rect),
}

pub fn main() -> GameResult {
    let (ref mut ctx, _) = ContextBuilder::new("foo", "bar")
        .build()
        .unwrap();
    let mut shapes: Vec<Shape> = Vec::new();
    let mut rng = Rand32::new(4); // Random number chosen by fair die roll
    {{example}}

    Ok(())
}
```

```rust,template-definition
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;

mod scope_hack {
    use super::*;

    pub enum Shape {
        Circle(mint::Point2<f32>, f32),
        Rectangle(graphics::Rect),
    }
}

use self::scope_hack::*;

{{example}}

fn main() {

}
```

```rust,template-definition-no-main
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;

{{example}}
```

```rust,template-shapes
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;
use ggez::event::*;

enum Shape {
    Circle(mint::Point2<f32>, f32),
    Rectangle(graphics::Rect),
}

struct State {
    shapes: Vec<Shape>,
}

impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
}

{{example}}

```
