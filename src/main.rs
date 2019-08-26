mod action;
mod app;
mod common;
mod dots;
mod effects;
mod physics;
mod scene;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
extern crate rusty_machine;

use piston::event_loop::*;
use piston::input::*;

use app::App;
use common::coord::Coord;
use common::threadpool::ThreadPool;
use scene::Scene;

fn main() {

    // Create a new game
    let mut app = App::new(ThreadPool::new(4), Scene::new(Coord { x: 50.0, y: 50.0 }, 10));
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