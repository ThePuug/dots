mod action;
mod app;
mod common;
mod dots;
mod effects;
mod physics;
mod scene;
mod intelligence;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use opengl_graphics::{GlGraphics,GlyphCache,OpenGL,TextureSettings};
use piston::event_loop::*;
use piston::input::*;
use piston::window::{WindowSettings};
use std::sync::{Arc,Mutex};
use std::sync::mpsc::{channel,Sender,Receiver};

use app::{App};
use common::coord::{Coord};
use common::threadpool::{ThreadPool};
use dots::{Dot,DotFactory};
use effects::{Effect};
use intelligence::random::{Intelligence,IntelligenceFactory};
use physics::{Physics};
use scene::{Scene};

fn main() {
    // DI'ish setup
    let (tx,rx): (Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,Receiver<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>) = channel();
    let open_gl_version = OpenGL::V3_2;
    let scene_size = Coord { x: 50.0, y: 50.0 };
    let scale = 10;
    let num_workers = 1;
    let thread_pool = ThreadPool::new(num_workers);
    let window = WindowSettings::new("Dots", [scene_size.x * scale as f64, scene_size.y * scale as f64])
        .exit_on_esc(true)
        .graphics_api(open_gl_version)
        .vsync(true)
        .build()
        .unwrap();
    let scene: Scene<Dot<Intelligence>> = Scene::new(scale);
    let gl = GlGraphics::new(open_gl_version);
    let intelligence_factory = IntelligenceFactory::new();
    let dot_factory = DotFactory::new(intelligence_factory,tx.clone());
    let physics = Physics::new(thread_pool, scene, tx.clone(), rx, dot_factory);
    let glyph_cache = Mutex::new(GlyphCache::new("assets/UbuntuMono-R.ttf", (), TextureSettings::new()).unwrap());

    // main loop
    let mut app = App::new(physics, window, gl, glyph_cache);
    let mut es = EventSettings::new();
    es.set_ups(12);
    es.set_max_fps(60);
    let mut events = Events::new(es);
    while let Some(e) = events.next(&mut app.window) {
        if let Some(r) = e.render_args() { app.render(&r); }
        if let Some(u) = e.update_args() { app.update(&u); }
    }
}