use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::input::{RenderArgs,UpdateArgs};
use piston::window::WindowSettings;
use std::sync::Arc;

use common::threadpool::ThreadPool;
use effects::{Effect,EffectType};
use physics::Physics;
use scene::Scene;

pub struct App {
    pub gl: GlGraphics, // OpenGL drawing backend.
    pub window: Window,
    pub physics: Physics
}

impl App {
    pub fn new(pool: ThreadPool,
               scene: Scene) -> App {
        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        let physics = Physics::new(pool, scene);

        // Create an Glutin window.
        let window: Window = WindowSettings::new("Dots",[physics.scene.size.x * physics.scene.scale as f64, physics.scene.size.y * physics.scene.scale as f64])
            .opengl(opengl)
            .exit_on_esc(true)
            .vsync(true)
            .build().unwrap();

        return App {
            gl: GlGraphics::new(opengl),
            window,
            physics
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |_c, gl| {
            clear([1.0,1.0,1.0,1.0], gl); // Clear the screen.
        });

        for (_,mutex) in self.physics.scene.dots.lock().unwrap().iter() {
            let it = mutex.lock().unwrap();
            let dot = rectangle::centered_square(
                it.pos.x * self.physics.scene.scale as f64, 
                it.pos.y * self.physics.scene.scale as f64, 
                0.5 * self.physics.scene.scale as f64);
            self.gl.draw(args.viewport(), |c, gl| {
                rectangle(it.color, dot, c.transform, gl); // draw it
            });
        }
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        self.physics.apply();
        self.physics.queue((Arc::new(Vec::new()),Arc::new(Effect {
            pos: None,
            typ: Some(EffectType::TICK),
            val: Some(1.0)
        })));
    }
}
