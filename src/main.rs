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

use opengl_graphics::{GlGraphics,GlyphCache,OpenGL,TextureSettings};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use std::collections::HashMap;
use std::sync::{Arc,Mutex};

use app::App;
use common::coord::Coord;
use common::threadpool::ThreadPool;
use dots::Dot;
use neuralnets::random::{NeuralNet,NeuralNetFactory};
use physics::Physics;
use scene::Scene;

fn main() {
    // DI'ish setup
    let open_gl_version = OpenGL::V3_2;
    let scene_size = Coord { x: 50.0, y: 50.0 };
    let scale = 10;
    let num_workers = 1;
    let dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot<NeuralNet>>>>>> = Arc::new(Mutex::new(HashMap::new()));
    let thread_pool = ThreadPool::new(num_workers);
    let scene = Scene::new(scene_size, scale, dots);
    let window = WindowSettings::new("Dots", [scene.size.x * scene.scale as f64, scene.size.y * scene.scale as f64])
        .exit_on_esc(true)
        .graphics_api(open_gl_version)
        .vsync(true)
        .build()
        .unwrap();
    let gl = GlGraphics::new(open_gl_version);
    let neural_net_factory = NeuralNetFactory{};
    let physics = Physics::new(thread_pool, scene, neural_net_factory);
    let glyph_cache = Mutex::new(GlyphCache::new("assets/UbuntuMono-R.ttf", (), TextureSettings::new()).unwrap());

    // Create a new game
    let mut app = App::new(physics, window, gl, glyph_cache);

    let mut es = EventSettings::new();
    es.set_ups(12);
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