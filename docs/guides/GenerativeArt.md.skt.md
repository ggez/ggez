```rust,skt-draw
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;
use rand::*;

enum Shape {{
    Circle(mint::Point2<f32>, f32),
    Rectangle(graphics::Rect),
}}

struct State {{
    shapes: Vec<Shape>,
}}

impl State {{
    fn new(_ctx: &mut Context) -> GameResult<Self> {{
        let s = State {{ shapes: Vec::new() }};
        Ok(s)
    }}
}}

impl event::EventHandler for State {{
    fn update(&mut self, _ctx: &mut Context) -> GameResult {{
        Ok(())
    }}

    {}
}}

pub fn main() -> GameResult {{
    Ok(())
}}
```

```rust,skt-expression
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;
use rand::*;

enum Shape {{
    Circle(mint::Point2<f32>, f32),
    Rectangle(graphics::Rect),
}}

pub fn main() -> GameResult {{
    let (ref mut ctx, _) = ContextBuilder::new("foo", "bar")
        .build()
        .unwrap();
    let mut shapes: Vec<Shape> = Vec::new();
    {}

    Ok(())
}}
```

```rust,skt-definition
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;

mod scope_hack {{  
    use super::*;

    pub enum Shape {{
        Circle(mint::Point2<f32>, f32),
        Rectangle(graphics::Rect),
    }}
}}

use self::scope_hack::*;

{}

fn main() {{

}}
```

```rust,skt-definition-no-main
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;

{}
```

```rust,skt-shapes
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;
use ggez::event::*;

enum Shape {{
    Circle(mint::Point2<f32>, f32),
    Rectangle(graphics::Rect),
}}

struct State {{
    shapes: Vec<Shape>,
}}

impl event::EventHandler for State {{
    fn update(&mut self, _ctx: &mut Context) -> GameResult {{
        Ok(())
    }}

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {{
        Ok(())
    }}
}}

{}

```