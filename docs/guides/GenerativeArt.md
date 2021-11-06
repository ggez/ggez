# 🎨 Generative Art

This is a guide to create a program that randomly generates shapes to display.

At the end of this guide you will:
*  Have used the [`graphics`](https://docs.rs/ggez/0.7.0/ggez/graphics/) module of `ggez` to draw shapes

## Project Setup

Create a new crate using `cargo new --bin generative_art` and `cd` into it.

Modify `Cargo.toml` to include `ggez` in the dependencies.

Just like in `Hello ggez!`, we're going to use a loop and a struct.
Let's start with this code in `src/main.rs`:

```rust,no_run
use ggez::*;

struct State {}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() {
    let state = State {};
    let cb = ggez::ContextBuilder::new("generative_art", "awesome_person");
    let (ctx, event_loop) = cb.build().unwrap();
    event::run(ctx, event_loop, state);
}
```

### ✔ Check Project Setup

Test to make sure everything is correct by running `cargo run`.

If there are no errors and you see a window you are good.

## [ggez::graphics](https://docs.rs/ggez/0.7.0/ggez/graphics/)

Glancing over the docs for [`ggez::graphics`](https://docs.rs/ggez/0.7.0/ggez/graphics/) you can see there is a lot of functionality there.
The basic shapes can be found in [`graphics::Mesh`](https://docs.rs/ggez/0.7.0/ggez/graphics/struct.Mesh.html#methods)

So which shapes are in `Mesh`?

* [line](https://docs.rs/ggez/0.7.0/ggez/graphics/struct.Mesh.html#method.new_line)
* [circle](https://docs.rs/ggez/0.7.0/ggez/graphics/struct.Mesh.html#method.new_circle)
* [ellipse](https://docs.rs/ggez/0.7.0/ggez/graphics/struct.Mesh.html#method.new_ellipse)
* [polygon](https://docs.rs/ggez/0.7.0/ggez/graphics/struct.Mesh.html#method.new_polygon)
* [rectangle](https://docs.rs/ggez/0.7.0/ggez/graphics/struct.Mesh.html#method.new_rectangle)

We are just going to touch on 2 shapes in this guide: the circle and the rectangle.

Additionally there are 2 other methods we will look at.
These 2 are used to show and erase the screen:

* [clear](https://docs.rs/ggez/0.7.0/ggez/graphics/fn.clear.html)
* [present](https://docs.rs/ggez/0.7.0/ggez/graphics/fn.present.html)

### ⚫ The Circle

Circles are represented by 2 pieces of information: origin, and radius.
Geometry, it's all coming back now.

Here is the code for a [circle](https://docs.rs/ggez/0.7.0/ggez/graphics/struct.Mesh.html#method.new_circle):
```rust,skt-expression,no_run
graphics::Mesh::new_circle(
    ctx,
    graphics::DrawMode::fill(),
    mint::Point2{x: 200.0, y: 300.0},
    100.0,
    0.1,
    graphics::Color::WHITE,
)?;
```

Now now now, hold on a second... I said 2 pieces of information!
Why did I just write like a million?!

Well, `ctx` is needed to tell `ggez` where you are drawing to.
`ctx` is what is passed into `update` and `draw` already.

[`graphics::DrawMode::fill()`](https://docs.rs/ggez/0.7.0/ggez/graphics/enum.DrawMode.html) is choosing between outlining the circle or filling it in.

Point, now here is one we expected.
This is the origin of the circle.
`mint::Point2{x: 200.0, y: 300.0}` locates the circle at `x: 200, y: 300`.

`100.0` is the radius of the circle. So this circle will be `200` pixels wide.

And `0.1` is the [tolerance](https://docs.rs/lyon_geom/0.15.3/lyon_geom/#flattening). Do not worry about this one for now. You can experiment with the number.

And that's how a circle is drawn!

#### ✔ Check Circle

Let's try it out with some quick code:
```rust,skt-draw,no_run
fn draw(&mut self, ctx: &mut Context) -> GameResult {
    graphics::clear(ctx, graphics::Color::BLACK);
    let circle = graphics::Mesh::new_circle(
        ctx,
        graphics::DrawMode::fill(),
        mint::Point2{x: 200.0, y: 300.0},
        100.0,
        0.1,
        graphics::Color::WHITE,
    )?;
    graphics::draw(ctx, &circle, graphics::DrawParam::default())?;
    graphics::present(ctx)?;
    Ok(())
}
```

If you see a circle on the screen when you run `cargo run` you're good!

### ⬛ The Rectangle

Rectangles are represented by 3 pieces of information: origin, width, and height.

Here is the code for a [rectangle](https://docs.rs/ggez/0.7.0/ggez/graphics/struct.Mesh.html#method.new_rectangle):
```rust,skt-expression,no_run
graphics::Mesh::new_rectangle(
    ctx,
    graphics::DrawMode::fill(),
    graphics::Rect::new(500.0, 250.0, 200.0, 100.0),
    graphics::Color::WHITE,
)?;
```

It might seem weird that there are less parameters for more required information than a circle,
but this is correct.

[`Rect`](https://docs.rs/ggez/0.7.0/ggez/graphics/struct.Rect.html) is a convenient type that represents a... rectangle!
[`graphics::Rect::new(500.0, 250.0, 200.0, 100.0)`](https://docs.rs/ggez/0.7.0/ggez/graphics/struct.Rect.html) positions the rectangle's top-left corner at `x: 500, y: 250` and
specifies `width: 200, height: 100`.

And that's how a rectangle is drawn!

#### ✔ Check Rectangle

Let's try it out with some quick code:
```rust,skt-draw,no_run
fn draw(&mut self, ctx: &mut Context) -> GameResult {
    graphics::clear(ctx, graphics::Color::BLACK);
    let rect = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(500.0, 250.0, 200.0, 100.0),
        graphics::Color::WHITE,
    )?;
    graphics::draw(ctx, &rect, graphics::DrawParam::default())?;
    graphics::present(ctx)?;
    Ok(())
}
```

If you see a rectangle on the screen when you run `cargo run`, you're good to go.

## 🎨 Random Shapes

Start rubbing the right side of your head.
This will get your creative jive fired up.

When the program starts up, we want to generate a bunch of shapes and draw them.

Let's modify state to contain a Circle and a Rectangle.
We will use [`enum`](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html), [`Vec`](https://doc.rust-lang.org/book/ch08-01-vectors.html), and [`match`](https://doc.rust-lang.org/book/ch06-02-match.html) Rust features to do this.

We need a place to store our shapes.

Modify your `State`:
```rust,skt-definition,no_run
struct State {
    shapes: Vec<Shape>,
}
```

We are saying we want a list of `Shape` values.
But what is the `Shape` type?
Let's create it!

Create the `Shape` enum:
```rust,skt-definition,no_run
enum Shape {
    Circle(mint::Point2<f32>, f32),
    Rectangle(graphics::Rect),
}
```
With this `enum`, we are saying each `Shape` can be either a `Circle`, or a `Rectangle`.

Store 2 `Shape` values in your `shapes` `Vec`.
```rust,skt-shapes,no_run
fn main() {
    let mut shapes = Vec::new();
    shapes.push(Shape::Rectangle(ggez::graphics::Rect::new(
        10.0,
        10.0,
        50.0,
        100.0,
    )));
    shapes.push(Shape::Circle(
        mint::Point2{x: 400.0, y: 40.0},
        30.0,
    ));
    let state = State { shapes: shapes };
    let c = conf::Conf::new();
    let (ctx, event_loop) = ContextBuilder::new("generative_art", "awesome_person")
        .default_conf(c)
        .build()
        .unwrap();
    event::run(ctx, event_loop, state);
}
```
We're using `Vec.push` to add 2 new values to our `Vec`.

But we still don't see anything...
You need to modify `draw` to illustrate your new `State`.
```rust,skt-draw,no_run
fn draw(&mut self, ctx: &mut Context) -> GameResult {
    graphics::clear(ctx, graphics::Color::BLACK);
    for shape in &self.shapes {
        // Make the shape...
        let mesh = match shape {
            &Shape::Rectangle(rect) => {
                graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, graphics::Color::WHITE)?
            }
            &Shape::Circle(origin, radius) => {
                graphics::Mesh::new_circle(ctx, graphics::DrawMode::fill(), origin, radius, 0.1, graphics::Color::WHITE)?
            }
        };

        // ...and then draw it.
        graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
    }
    graphics::present(ctx)?;
    Ok(())
}
```

That's great.
We see 2 shapes on the screen now.

Let's go another step further.
We are going to allow the computer to randomly generate art for us.
Random numbers can be generated using the [rand crate](https://docs.rs/rand/0.8.3/rand/).

We don't want 2 shapes fixed in location.
We want 2 shapes randomly generated!

Include your favorite RNG in your `Cargo.toml`:
```toml
[dependencies]
ggez = "0.7"
oorandom = "11"
```

At the top of `main.rs` add a `use` statement for the RNG:
```rust,skt-definition,no_run
use oorandom::Rand32;
```

Change how the 2 shapes are created to include randomness:
```rust,skt-expression,no_run
    let mut rng = Rand32::new(4); // Random number chosen by fair die roll
shapes.push(Shape::Rectangle(ggez::graphics::Rect::new(
    rng.rand_float() * 800.0,
    rng.rand_float() * 600.0,
    rng.rand_float() * 800.0,
    rng.rand_float() * 600.0,
)));
shapes.push(Shape::Circle(
    mint::Point2{
        x: rng.rand_float() * 800.0,
        y: rng.rand_float() * 600.0,
    },
    rng.rand_float() * 300.0,
));
```
Run `cargo run` a couple times and see how the shapes move around and change shape.
We've now allowed our computer to generate some simple art for us.
But only 2 shapes?
Surely we can do better!

Modify your `main.rs`:
```rust,skt-expression,no_run
let mut shapes = Vec::new();
for _ in 0..8 {
    if rng.rand_i32() >= 0 {
        shapes.push(Shape::Rectangle(ggez::graphics::Rect::new(
            rng.rand_float() * 800.0,
            rng.rand_float() * 600.0,
            rng.rand_float() * 800.0,
            rng.rand_float() * 600.0,
        )));
    } else {
        shapes.push(Shape::Circle(
            mint::Point2{
                x: rng.rand_float() * 800.0,
                y: rng.rand_float() * 600.0,
            },
            rng.rand_float() * 300.0,
        ));
    }
}
```
Now we are creating 8 shapes of random type, size, and position.

Run `cargo run` a couple more times.  Add the `getrandom` crate or some
other source of true randomness to seed the RNG.
Bask in the expressiveness and technique of your computer.
Print it out, put it on a canvas, and sell it to your local museum.

## 💪 Challenges

Optional activities for those that want more:
 * Randomize colors
 * Include more shapes: lines, polygons, ellipse
 * Draw a new shape every 5 seconds instead of upfront
