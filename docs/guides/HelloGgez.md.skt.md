```rust,skt-draw
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use anyhow;
use ggez::*;
use ggez::graphics::*;
use ggez::event::*;
use std::time::Duration;

struct State {{
    pub dt: Duration,
}}

impl EventHandler for State {{
    fn update(&mut self, _ctx: &mut Context) -> anyhow::Result<()> {{
        Ok(())
    }}

    {}
}}

pub fn main() -> anyhow::Result<()> {{
    Ok(())
}}
```

```rust,skt-update
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use anyhow;
use ggez::*;
use ggez::graphics::*;

use std::time::Duration;

struct State {{
    pub dt: Duration,
}}

impl event::EventHandler for State {{
    fn draw(&mut self, _ctx: &mut Context) -> anyhow::Result<()> {{
        Ok(())
    }}

    {}
}}

pub fn main() -> anyhow::Result<()> {{
    Ok(())
}}
```

```rust,skt-expression
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]


use anyhow;
use ggez::*;
use ggez::graphics::*;
use ggez::event::*;
use rand::*;
use std::time::Duration;

struct State {{
    pub dt: Duration,
}}

impl EventHandler for State {{
    fn update(&mut self, _c: &mut Context) -> anyhow::Result<()> {{
        Ok(())
    }}

    fn draw(&mut self, _c: &mut Context) -> anyhow::Result<()> {{
        Ok(())
    }}
}}

pub fn main() -> anyhow::Result<()> {{
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("foo", "bar")
        .build()
        .unwrap();
    let state = &mut State {{ dt: Duration::default() }};
    {}

    Ok(())
}}
```

```rust,skt-definition
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use anyhow;
use ggez::*;
use ggez::graphics::*;

{}

fn main() {{

}}
```

```rust,skt-definition-no-main
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use anyhow;
use ggez::*;
use ggez::graphics::*;

struct State {{

}}

{}
```
