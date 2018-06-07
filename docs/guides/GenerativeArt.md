# 🎨 Generative Art

This is a guide to create a program that randomly generates shapes to display.

At the end of this guide you will:
*  Have used the [`graphics`](https://docs.rs/ggez/0.4.0/ggez/graphics/index.html) module of `ggez` to draw shapes

## Project Setup

Create a new crate using `cargo new --bin generative_art` and `cd` into it.

Modify `Cargo.toml` to include `ggez` in the dependencies.

Just like in `Hello ggez!`, we're going to use a loop and a struct.
Let's start with this code in `src/main.rs`:
```rust
extern crate ggez;
use ggez::*;

struct State {
}

impl ggez::event::EventHandler for State {
  fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
      Ok(())
  }
  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
      Ok(())
  }
}

fn main() {
    let state = &mut State { };
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("generative_art", "awesome_person", c).unwrap();
    event::run(ctx, state).unwrap();
}
```

### ✔ Check Project Setup

Test to make sure everything is correct by running `cargo run`.

If there are no errors and you see a window you are good.

## [`ggez::graphics`](https://docs.rs/ggez/0.4.0/ggez/graphics/index.html)

Glancing over the docs for [`ggez::graphics`](https://docs.rs/ggez/0.4.0/ggez/graphics/index.html) you can see there is a lot of functionality there.

So which shapes are in graphics?

* [circle](https://docs.rs/ggez/0.4.0/ggez/graphics/fn.circle.html)
* [ellipse](https://docs.rs/ggez/0.4.0/ggez/graphics/fn.ellipse.html)
* [line](https://docs.rs/ggez/0.4.0/ggez/graphics/fn.line.html)
* [points](https://docs.rs/ggez/0.4.0/ggez/graphics/fn.points.html)
* [polygon](https://docs.rs/ggez/0.4.0/ggez/graphics/fn.polygon.html)
* [rectangle](https://docs.rs/ggez/0.4.0/ggez/graphics/fn.rectangle.html)

We are just going to touch on 2 shapes in this guide: the circle and the rectangle.

Additionally there are 2 other methods we will look at.
These 2 are used to show and erase the screen:

* [clear](https://docs.rs/ggez/0.4.0/ggez/graphics/fn.clear.html)
* [present](https://docs.rs/ggez/0.4.0/ggez/graphics/fn.present.html)

### ⚫ The Circle

Circles are represented by 2 pieces of information: origin, and radius.
Geometry, it's all coming back now.

Here is the code for a [circle](https://docs.rs/ggez/0.4.0/ggez/graphics/fn.circle.html):
```rust
graphics::circle(
    ctx,
    graphics::DrawMode::Fill,
    graphics::Point2::new(200.0, 300.0),
    100.0,
    0.1,
);
```

Now now now, hold on a second... I said 2 pieces of information!
Why did I just write like a million?!

Well, `ctx` is needed to tell `ggez` where you are drawing to.
`ctx` is what is passed into `update` and `draw` already.

[`graphics::DrawMode::Fill`](https://docs.rs/ggez/0.4.0/ggez/graphics/enum.DrawMode.html) is choosing between outlining the circle or filling it in.

Point, now here is one we expected.
This is the origin of the circle.
[`graphics::Point2::new(200.0, 300.0)`](https://docs.rs/ggez/0.4.0/ggez/graphics/type.Point2.html) locates the circle at `x: 200, y: 300`.

`100.0` is the radius of the circle. So this circle will be `200` pixels wide.

And `0.1` is the [tolerance](https://docs.rs/lyon_geom/0.9.0/lyon_geom/#flattening). Do not worry about this one for now. You can experiment with the number.

And that's how a circle is drawn!

#### ✔ Check Circle

Let's try it out with some quick code:
```rust
fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
  graphics::clear(ctx);
  graphics::circle(
      ctx,
      graphics::DrawMode::Fill,
      graphics::Point2::new(200.0, 300.0),
      100.0,
      0.1,
  );
  graphics::present(ctx);
  Ok(())
}
```

If you see a circle on the screen when you run `cargo run` you're good!

### ⬛ The Rectangle

Rectangles are represented by 3 pieces of information: origin, width, and height.

Here is the code for a [rectangle](https://docs.rs/ggez/0.4.0/ggez/graphics/fn.rectangle.html):
```rust
graphics::rectangle(
    ctx,
    graphics::DrawMode::Fill,
    graphics::Rect::new(500.0, 250.0, 200.0, 100.0),
);
```

It might seem weird that there are less parameters for more required information than a circle,
but this is correct.

[`Rect`](https://docs.rs/ggez/0.4.0/ggez/graphics/struct.Rect.html) is a convenient type that represents a... rectangle!
[`graphics::Rect::new(500.0, 250.0, 200.0, 100.0)`](https://docs.rs/ggez/0.4.0/ggez/graphics/struct.Rect.html) positions the rectangle's top-left corner at `x: 500, y: 250` and
specifies `width: 200, height: 100`.

And that's how a rectangle is drawn!

#### ✔ Check Rectangle

Let's try it out with some quick code:
```rust
fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
  graphics::clear(ctx);
  graphics::rectangle(
      ctx,
      graphics::DrawMode::Fill,
      graphics::Rect::new(500.0, 250.0, 200.0, 100.0),
  );
  graphics::present(ctx);
  Ok(())
}
```

If you see a rectangle on the screen when you run `cargo run`, you're good to go.

## 🎨 Random Shapes

Start rubbing the right side of your head.
This will get your creative jive fired up.

When the program starts up, we want to generate a bunch of shapes and draw them.

Let's modify state to contain a Circle and a Rectangle.
We will use [`enum`](https://doc.rust-lang.org/book/second-edition/ch06-01-defining-an-enum.html), [`Vec`](https://doc.rust-lang.org/book/second-edition/ch08-01-vectors.html), and [`match`](https://doc.rust-lang.org/book/second-edition/ch06-02-match.html) Rust features to do this.

We need a place to store our shapes.

Modify your `State`:
```rust
struct State {
    shapes: Vec<Shape>,
}
```

We are saying we want a list of `Shape` values.
But what is the `Shape` type?
Let's create it!

Create the `Shape` enum:
```rust
enum Shape {
    Circle(graphics::Point2, f32),
    Rectangle(graphics::Rect),
}
```
With this `enum`, we are saying each `Shape` can be either a `Circle`, or a `Rectangle`.

Store 2 `Shape` values in your `shapes` `Vec`.
```rust
fn main() {
    let mut shapes = Vec::new();
    shapes.push(Shape::Rectangle(ggez::graphics::Rect::new(
        10.0,
        10.0,
        50.0,
        100.0,
    )));
    shapes.push(Shape::Circle(
        ggez::graphics::Point2::new(400.0, 40.0),
        30.0,
    ));
    let state = &mut State { shapes: shapes };
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("generative_art", "awesome_person", c).unwrap();
    event::run(ctx, state).unwrap();
}
```
We're using `Vec.push` to add 2 new values to our `Vec`.

But we still don't see anything...
You need to modify `draw` to illustrate your new `State`.
```rust
fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx);
    for shape in &self.shapes {
        match shape {
            &Shape::Rectangle(rect) => {
                graphics::rectangle(ctx, graphics::DrawMode::Fill, rect).unwrap();
            }
            &Shape::Circle(origin, radius) => {
                graphics::circle(ctx, graphics::DrawMode::Fill, origin, radius, 0.1).unwrap();
            }
        }
    }
    graphics::present(ctx);
    Ok(())
}
```

That's great.
We see 2 shapes on the screen now.

Let's go another step further.
We are going to allow the computer to randomly generate art for us.
Random numbers can be generated using the [rand crate](https://doc.rust-lang.org/rand/rand/index.html).

We don't want 2 shapes fixed in location.
We want 2 shapes randomly generated!

Include `rand` in your `Cargo.toml`:
```toml
[dependencies]
ggez = "0.4"
rand = "0.4.2"
```

At the top of `main.rs`, `extern crate` and `use` `rand`:
```rust
extern crate rand;
use rand::{thread_rng, Rng};
```

Change how the 2 shapes are created to include randomness:
```rust
shapes.push(Shape::Rectangle(ggez::graphics::Rect::new(
    thread_rng().gen_range(0.0, 800.0),
    thread_rng().gen_range(0.0, 600.0),
    thread_rng().gen_range(0.0, 800.0),
    thread_rng().gen_range(0.0, 600.0),
)));
shapes.push(Shape::Circle(
    ggez::graphics::Point2::new(
        thread_rng().gen_range(0.0, 800.0),
        thread_rng().gen_range(0.0, 600.0),
    ),
    thread_rng().gen_range(0.0, 300.0),
));
```
Run `cargo run` a couple times and see how the shapes move around and change shape.
We've now allowed our computer to generate some simple art for us.
But only 2 shapes?
Surly we can do better!

Modify your `main.rs`:
```rust
let mut shapes = Vec::new();
for _ in 0..8 {
    if thread_rng().gen_range(0, 2) % 2 == 0 {
        shapes.push(Shape::Rectangle(ggez::graphics::Rect::new(
            thread_rng().gen_range(0.0, 800.0),
            thread_rng().gen_range(0.0, 600.0),
            thread_rng().gen_range(0.0, 800.0),
            thread_rng().gen_range(0.0, 600.0),
        )));
    } else {
        shapes.push(Shape::Circle(
            ggez::graphics::Point2::new(
                thread_rng().gen_range(0.0, 800.0),
                thread_rng().gen_range(0.0, 600.0),
            ),
            thread_rng().gen_range(0.0, 300.0),
        ));
    }
}
```
Now we are creating 8 shapes of random type, size, and position.

run `cargo run` a couple more times.
Bask in the expressiveness and technique of your computer.
Print it out, put it on a canvas, and sell it to your local museum.

## 💪 Challenges

Optional activities for those that want more:
* Randomize colors
* Include more shapes: lines, polygons, ellipse
* Draw a new shape every 5 seconds instead of upfront
