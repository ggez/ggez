```rust,skt-draw
#![allow(unused_imports)]
#![allow(dead_code)]
extern crate ggez;

use ggez::event;
use ggez::graphics::{{self, Drawable}};
use ggez::{{Context, GameResult}};

struct State {{
}}

impl State {{
    fn new(_ctx: &mut Context) -> GameResult<Self> {{
        let s = State {{ }};
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