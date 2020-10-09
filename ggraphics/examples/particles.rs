// Suggested logging level for debugging:
// env RUST_LOG=info cargo run

use ggraphics::*;
use glam::{self, Mat4, Quat, Vec2, Vec3};
use glow;
use oorandom;
use winit;

use std::rc::Rc;
use std::time::Duration;

struct Particle {
    pos: Vec2,
    vel: Vec2,
    rot: f32,
    rvel: f32,
    life: f32,
}

impl Particle {}

struct GameState {
    ctx: Rc<GlContext>,
    rng: oorandom::Rand32,
    particles: Vec<Particle>,
    passes: Vec<RenderPass>,
    pipelines: Vec<Box<dyn Pipeline<BatchType = MeshBatch>>>,

    particle_texture: Texture,
    particle_mesh: Mesh,
}

impl GameState {
    fn update_mesh_pipeline(&mut self) {
        // Mesh pipeline is always pipeline 0 here, for reasons
        for batch in self.pipelines[0].batches().iter_mut() {
            batch.clear();
            for particle in self.particles.iter() {
                let mut q = MeshInstance::empty();
                let trans = Mat4::from_scale_rotation_translation(
                    Vec3::new(0.1, 0.1, 0.1),
                    Quat::from_rotation_z(particle.rot * 2.0 * std::f32::consts::PI),
                    Vec3::new(
                        particle.pos.x() * 2.0 - 1.0,
                        particle.pos.y() * 2.0 - 1.0,
                        0.0,
                    ),
                );
                q.model_transform = trans.to_cols_array();
                batch.add(q);
            }
        }
    }

    pub fn new(gl: glow::Context) -> Self {
        let ctx = Rc::new(GlContext::new(gl));
        let mut passes = vec![];
        let mut pipelines: Vec<Box<dyn Pipeline<BatchType = MeshBatch> + 'static>> = vec![];
        let mut rng = oorandom::Rand32::new(12345);
        let (particle_mesh, particle_texture) = unsafe {
            let particle_texture = {
                let image_bytes = include_bytes!("../src/data/wabbit_alpha.png");
                let image_rgba = image::load_from_memory(image_bytes).unwrap().to_rgba();
                let (w, h) = image_rgba.dimensions();
                let image_rgba_bytes = image_rgba.into_raw();
                TextureHandle::new(&ctx, &image_rgba_bytes, w as usize, h as usize).into_shared()
            };
            // Render that texture to the screen
            let shader = GlContext::default_shader(&ctx);
            let projection = Mat4::orthographic_rh_gl(-1.0, 1.0, -1.0, 1.0, 1.0, -1.0);
            /*
            let mut pipeline = QuadPipeline::new(ctx.clone(), shader.clone(), projection);
            pipeline.new_drawcall(particle_texture.clone(), SamplerSpec::default());
            pipelines.push(Box::new(pipeline));
            */

            // Make pipeline for meshes.
            // TODO: This at least makes the resource dependencies clear:
            // making a Mesh requires access to the shader (currently), since
            // that Mesh contains the VAO which associates the buffers with the
            // shader locations.
            // But the shader itself is owned by the Pipeline.
            // So what I THINK should happen is that the Mesh itself is JUST the
            // VBO's, then the Batch contains the VAO that associates them to
            // the shader, so that happens when the Batch is created.
            // Yeah, that works.
            let particle_mesh = {
                let verts = vec![
                    Vertex {
                        pos: [0.0, 0.0, 0.0, 1.0],
                        color: [0.0, 0.0, 1.0, 1.0],
                        uv: [0.0, 0.0],
                    },
                    Vertex {
                        pos: [1.0, 0.0, 0.0, 1.0],
                        color: [1.0, 0.0, 1.0, 1.0],
                        uv: [1.0, 0.0],
                    },
                    Vertex {
                        pos: [0.0, 1.0, 0.0, 1.0],
                        color: [0.0, 1.0, 1.0, 1.0],
                        uv: [0.0, 1.0],
                    },
                    Vertex {
                        pos: [1.0, 1.0, 0.0, 1.0],
                        color: [1.0, 1.0, 0.0, 1.0],
                        uv: [1.0, 1.0],
                    },
                ];
                let indices = vec![0, 1, 2, 2, 3, 1];
                MeshHandle::new(&ctx, &verts, &indices).into_shared()
            };

            let mut mesh_pipeline = MeshPipeline::new(ctx.clone(), shader.clone());
            let mut batch = MeshBatch::new(
                ctx.clone(),
                particle_texture.clone(),
                particle_mesh.clone(),
                SamplerSpec::default(),
                &mesh_pipeline,
            );
            for _ in 0..100 {
                let mut q = MeshInstance::empty();
                let trans = Mat4::from_scale_rotation_translation(
                    Vec3::new(0.1, 0.1, 0.1),
                    Quat::from_rotation_z(rng.rand_float() * 2.0 * std::f32::consts::PI),
                    Vec3::new(
                        rng.rand_float() * 2.0 - 1.0,
                        rng.rand_float() * 2.0 - 1.0,
                        0.0,
                    ),
                );
                q.model_transform = trans.to_cols_array();
                batch.add(q);
            }
            mesh_pipeline.batches.push(batch);
            pipelines.push(Box::new(mesh_pipeline));

            // Make render pass rendering to screen
            let screen_pass = RenderPass::new_screen(&*ctx, 800, 600);
            passes.push(screen_pass);
            (particle_mesh, particle_texture)
        };

        Self {
            ctx,
            rng,
            particles: vec![],
            passes,
            pipelines: pipelines,

            particle_mesh,
            particle_texture,
        }
    }

    pub fn add_particles(&mut self, source_pt: Vec2) {
        const PARTICLE_COUNT: usize = 10000;
        for _ in 0..PARTICLE_COUNT {
            let particle = Particle {
                pos: source_pt,
                vel: glam::vec2(
                    -0.005 + self.rng.rand_float() * 0.01,
                    0.03 + self.rng.rand_float() * 0.005,
                ),
                rot: 0.0,
                rvel: -0.05 + self.rng.rand_float() * 0.1,
                life: 1.5,
            };
            self.particles.push(particle);
        }
    }

    pub fn update(&mut self, frametime: Duration) -> usize {
        return 0;
        // Update all our particle state
        for particle in &mut self.particles {
            particle.life -= frametime.as_secs_f32();
            particle.pos += particle.vel;
            particle.rot += particle.rvel;
            // gravity
            particle.vel -= glam::vec2(0.0, 0.0005);

            // Bounce off bottom and sides of screen
            // Currently, 0,0 to 1,1
            if particle.pos.y() < 0.0 {
                particle.vel.set_y(-particle.vel.y());
            }
            if particle.pos.x() < 0.0 || particle.pos.x() > 1.0 {
                particle.vel.set_x(-particle.vel.x());
            }
        }
        /*
        // Clean out dead particles.
        self.particles.retain(|p| p.life > 0.0);
        // Copy particles into draw call, since they've changed.
        // If our update framerate were faster than our drawing
        // frame rate, we'd want to do this on draw rather than update.
        for pipeline in self.pipelines.iter_mut() {
            for drawcall in pipeline.drawcalls_mut() {
                // Copy all our particles into the draw call
                drawcall.clear();
                for _particle in &self.particles {
                    //let q = particle.into_quaddata();
                    let q = MeshInstance::empty();
                    drawcall.add(q);
                }
            }
        }
        */
        self.particles.len()
    }

    /// Sets the viewport for the render pass.
    /// Negative numbers are valid, see `glViewport` for the
    /// math behind it.
    pub fn set_screen_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let pass = self
            .passes
            .last_mut()
            .expect("set_screen_viewport() requires a render pass to function on");
        if pass.is_screen() {
            pass.set_viewport(x, y, w, h);
        } else {
            panic!("Last render pass is not rendering to screen, aiee!");
        }
    }
}

trait Window {
    fn request_redraw(&self);
    fn swap_buffers(&self);
    // TODO: Resize
}

/// Used for desktop
#[cfg(not(target_arch = "wasm32"))]
impl Window for glutin::WindowedContext<glutin::PossiblyCurrent> {
    fn request_redraw(&self) {
        self.window().request_redraw();
    }
    fn swap_buffers(&self) {
        self.swap_buffers().unwrap();
    }
}

/// Used for wasm
#[cfg(target_arch = "wasm32")]
impl Window for winit::window::Window {
    fn request_redraw(&self) {
        self.request_redraw();
    }
    fn swap_buffers(&self) {
        /*
        let msg = format!("swapped buffers");
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
        */
    }
}

fn mainloop(
    gl: glow::Context,
    event_loop: winit::event_loop::EventLoop<()>,
    window: impl Window + 'static,
) {
    use instant::Instant;
    use log::*;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::ControlFlow;
    let mut state = GameState::new(gl);
    state.add_particles(glam::vec2(0.5, 0.1));
    let (vend, rend, vers, shader_vers) = state.ctx.get_info();
    info!(
        "GL context created.
  Vendor: {}
  Renderer: {}
  Version: {}
  Shader version: {}",
        vend, rend, vers, shader_vers
    );

    // EVENT LOOP
    {
        let mut frames = 0;
        let target_dt = Duration::from_micros(10_000);
        let mut last_frame = Instant::now();
        let mut next_frame = last_frame + target_dt;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::WaitUntil(next_frame);
            //*control_flow = ControlFlow::Poll;
            match event {
                Event::LoopDestroyed => {
                    info!("Event::LoopDestroyed!");
                    return;
                }
                Event::MainEventsCleared => {
                    let now = Instant::now();
                    let dt = now - last_frame;
                    if dt >= target_dt {
                        /*
                        #[cfg(target_arch = "wasm32")]
                        {
                            let msg = format!("Events cleared: {:?}, target: {:?}", dt, target_dt);
                            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
                        }
                        */
                        let num_objects = state.update(dt);
                        last_frame = now;
                        next_frame = now + target_dt;

                        frames += 1;
                        const FRAMES: u32 = 100;
                        if frames % FRAMES == 0 {
                            let fps = 1.0 / dt.as_secs_f64();
                            info!("{} objects, {:.03} fps", num_objects, fps);
                        }
                        window.request_redraw();
                    }
                }
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        info!("WindowEvent::Resized: {:?}", physical_size);
                        state.set_screen_viewport(
                            0,
                            0,
                            physical_size.width as i32,
                            physical_size.height as i32,
                        );
                        //windowed_context.resize(logical_size.to_physical(dpi_factor));
                    }
                    WindowEvent::CloseRequested => {
                        info!("WindowEvent::CloseRequested");
                        // Don't need to drop Context explicitly,
                        // it'll happen when we exit.
                        *control_flow = ControlFlow::Exit
                    }
                    WindowEvent::MouseInput {
                        button: winit::event::MouseButton::Left,
                        state: winit::event::ElementState::Pressed,
                        ..
                    }
                    /* These don't seem to actually work on mobile
                    WindowEvent::TouchpadPressure { .. } | WindowEvent::Touch(_) */
                    => {
                        // FUCJKLFSd;jflk;jds
                        // Winit doesn't actually give you a position with clicks.
                        state.add_particles(glam::vec2(0.5, 0.1));
                        println!("Total particles: {}", state.particles.len());
                    }
                    _ => (),
                },
                Event::RedrawRequested(_) => {
                    state.update_mesh_pipeline();
                    for pass in state.passes.iter_mut() {
                        pass.draw(&*state.ctx, Some((0.1, 0.2, 0.3, 1.0)), &state.pipelines);
                    }
                    window.swap_buffers();
                }
                _ => (),
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn run_wasm() {
    console_error_panic_hook::set_once();
    use winit::platform::web::WindowExtWebSys;
    let event_loop = winit::event_loop::EventLoop::new();
    let win = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .with_title("Heckin' winit")
        .build(&event_loop)
        .unwrap();

    let document = web_sys::window()
        .expect("Failed to obtain window")
        .document()
        .expect("Failed to obtain document");

    // Shove winit's canvas into the document
    document
        .body()
        .expect("Failed to obtain body")
        .append_child(&win.canvas())
        .unwrap();

    // Wire winit's context into glow
    let gl = {
        use wasm_bindgen::JsCast;
        let webgl2_context = win
            .canvas()
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap();
        glow::Context::from_webgl2_context(webgl2_context)
    };

    mainloop(gl, event_loop, win);
}

#[cfg(not(target_arch = "wasm32"))]
fn run_glutin() {
    use log::*;
    pretty_env_logger::init();
    // CONTEXT CREATION
    unsafe {
        // Create a context from a glutin window on non-wasm32 targets
        let (gl, event_loop, windowed_context) = {
            let el = glutin::event_loop::EventLoop::new();
            let wb = glutin::window::WindowBuilder::new()
                .with_title("Hello triangle!")
                .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));
            let windowed_context = glutin::ContextBuilder::new()
                .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (2, 0)))
                //.with_gl(glutin::GlRequest::GlThenGles {
                //    opengl_version: (4, 3),
                //    opengles_version: (2, 0),
                //})
                .with_gl_profile(glutin::GlProfile::Core)
                .with_vsync(true)
                .build_windowed(wb, &el)
                .unwrap();
            let windowed_context = windowed_context.make_current().unwrap();
            let context = glow::Context::from_loader_function(|s| {
                windowed_context.get_proc_address(s) as *const _
            });
            (context, el, windowed_context)
        };
        trace!("Window created");

        // GL SETUP
        mainloop(gl, event_loop, windowed_context);
    }
}

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    run_wasm();
    #[cfg(not(target_arch = "wasm32"))]
    run_glutin();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn wasm_main() {
    main();
}
