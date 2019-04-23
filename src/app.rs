use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::input::{RenderArgs,UpdateArgs};
use piston::window::WindowSettings;

use common::coord::Coord;
use effects::{Effect,EffectType};
use scene::Scene;

pub struct App {
    pub gl: GlGraphics, // OpenGL drawing backend.
    pub window: Window,
    pub scene: Scene
}

impl App {
    pub fn new() -> App {
        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create the scene
        let scene = Scene::new(Coord { x: 50.0, y: 50.0 }, 16);

        // Create an Glutin window.
        let window: Window = WindowSettings::new("Dots",[scene.size.x * scene.scale as f64, scene.size.y * scene.scale as f64])
            .opengl(opengl)
            .exit_on_esc(true)
            .vsync(true)
            .build().unwrap();

        return App {
            gl: GlGraphics::new(opengl),
            window,
            scene
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |_c, gl| {
            clear([1.0,1.0,1.0,1.0], gl); // Clear the screen.
        });

        for (_,mutex) in self.scene.physics.dots.lock().unwrap().iter() {
            let it = mutex.lock().unwrap();
            let dot = rectangle::centered_square(it.pos.x * self.scene.scale as f64, it.pos.y * self.scene.scale as f64, 0.5 * self.scene.scale as f64);
            self.gl.draw(args.viewport(), |c, gl| {
                rectangle(it.color, dot, c.transform, gl); // draw it
            });
        }
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        self.scene.physics.tx.send(Effect {
            pos: None,
            effect: EffectType::TICK,
            intensity: 1.0
        });
    }
}
