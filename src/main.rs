mod action;
mod app;
mod common;
mod dots;
mod effects;
mod physics;
mod scene;
mod neuralnets;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;

use app::App;
use common::coord::Coord;
use common::threadpool::ThreadPool;
use dots::Dot;
use neuralnets::random::{NeuralNet,NeuralNetFactory};
use physics::Physics;
use scene::Scene;

fn main() {
    // DI'ish setup
    let version = OpenGL::V3_2;
    let coord = Coord { x: 50.0, y: 50.0 };
    let scale = 10;
    let num_workers = 4;
    let dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot<NeuralNet>>>>>> = Arc::new(Mutex::new(HashMap::new()));
    let threadpool = ThreadPool::new(num_workers);
    let scene = Scene::new(coord, scale, dots);
    let window = WindowSettings::new("Dots", [scene.size.x * scene.scale as f64, scene.size.y * scene.scale as f64])
        .exit_on_esc(true)
        .vsync(true)
        .build()
        .unwrap();
    let gl = GlGraphics::new(version);
    let neuralnetfactory = NeuralNetFactory{};
    let physics = Physics::new(threadpool, scene, neuralnetfactory);

    // Create a new game
    let mut app = App::new(physics, window, gl);

    let mut es = EventSettings::new();
    es.set_ups(120);
    es.set_max_fps(60);
    let mut events = Events::new(es);
    while let Some(e) = events.next(&mut app.window) {

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