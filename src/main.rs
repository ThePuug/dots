mod dots;
mod common;
mod action;
mod scene;
mod effects;
mod physics;

extern crate rand;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use dots::Dot;
use common::coord::Coord;
use scene::Scene;
use physics::Engine;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    scene: Scene
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |_c, gl| {
            clear([1.0,1.0,1.0,1.0], gl); // Clear the screen.
        });

        for i in &self.scene.dots {
            let dot = rectangle::centered_square(i.pos.x * self.scene.scale as f64, i.pos.y * self.scene.scale as f64, 0.5 * self.scene.scale as f64);
            self.gl.draw(args.viewport(), |c, gl| {
                rectangle(i.color, dot, c.transform, gl); // draw it
            });
        }
    }

    fn update(&mut self, _args: &UpdateArgs) {
        for mut it in &self.scene.dots { it.tick(); }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // create the physics engine
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut engine = Engine::new(tx,rx);
        engine.start();
    });

    // create the origin of life
    let dot = Arc::new(Dot {
        pos: Coord{x: 100.0, y:100.0},
        color:[0.0,0.0,0.0,1.0],
        tx: tx.clone()});

    // Create the scene
    let mut scene = Scene::new(Coord { x: 200.0, y: 200.0 }, 3);
    scene.dots.push(dot);

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Dots",[scene.size.x * scene.scale as f64, scene.size.y * scene.scale as f64])
        .opengl(opengl)
        .exit_on_esc(true)
        .vsync(true)
        .build().unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        scene: scene
    };

    let mut es = EventSettings::new();
    es.set_ups(120);
    es.set_max_fps(60);
    let mut events = Events::new(es);
    while let Some(e) = events.next(&mut window) {

        // render frame
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        // game frame
        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}