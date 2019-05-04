use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::input::{RenderArgs,UpdateArgs};
use piston::window::WindowSettings;
use std::collections::HashMap;
use std::sync::{Arc,Mutex};

use common::coord::Coord;
use dots::Dot;
use effects::{Effect,EffectType};
use scene::Scene;

pub struct App {
    pub gl: GlGraphics, // OpenGL drawing backend.
    pub window: Window,
    pub scene: Scene,
    dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>>
}

impl App {
    pub fn new() -> App {
        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create the dots
        let dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>> = Arc::new(Mutex::new(HashMap::new()));

        let scene;
        {
            // Create the scene
            let dots = dots.clone();
            scene = Scene::new(Coord { x: 50.0, y: 50.0 }, 16, dots);
        }

        // Create an Glutin window.
        let window: Window = WindowSettings::new("Dots",[scene.size.x * scene.scale as f64, scene.size.y * scene.scale as f64])
            .opengl(opengl)
            .exit_on_esc(true)
            .vsync(true)
            .build().unwrap();

        return App {
            gl: GlGraphics::new(opengl),
            window,
            scene,
            dots
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |_c, gl| {
            clear([1.0,1.0,1.0,1.0], gl); // Clear the screen.
        });

        for (_,mutex) in self.dots.lock().unwrap().iter() {
            let it = mutex.lock().unwrap();
            let dot = rectangle::centered_square(it.pos.x * self.scene.scale as f64, it.pos.y * self.scene.scale as f64, 0.5 * self.scene.scale as f64);
            self.gl.draw(args.viewport(), |c, gl| {
                rectangle(it.color, dot, c.transform, gl); // draw it
            });
        }
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        match self.scene.physics.tx.send((Arc::new(Vec::new()),Arc::new(Effect {
            pos: None,
            typ: Some(EffectType::TICK),
            val: Some(1.0)
        }))) {
            Ok(_) => {},
            Err(err) => println!("{}",err)
        }
    }
}
