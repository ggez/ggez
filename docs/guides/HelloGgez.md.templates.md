```rust,template-draw
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;
use ggez::event::*;
use std::time::Duration;

struct State {
    pub dt: Duration,
}

impl EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    {{example}}
}

pub fn main() -> GameResult {
    Ok(())
}
```

```rust,template-update
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ggez::*;
use ggez::graphics::*;

use std::time::Duration;

struct State {
    pub dt: Duration,
}

impl event::EventHandler for State {
    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
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
use ggez::event::*;
use rand::*;
use std::time::Duration;

struct State {
    pub dt: Duration,
}

impl EventHandler for State {
    fn update(&mut self, _c: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, _c: &mut Context) -> GameResult {
        Ok(())
    }
}

pub fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("foo", "bar")
        .build()
        .unwrap();
    let state = State { dt: Duration::default() };
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

struct State {

}

{{example}}
```
