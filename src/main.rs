mod action;
mod app;
mod common;
mod dots;
mod effect;
mod scene;

use crate::app::App;
use crate::common::coord::Coord;
use crate::effect::Effect;
use crate::scene::Scene;

use common::dna::Dna;
use common::dna::SIZE;
use dots::DotFactory;
use flume::{unbounded, Receiver, Sender};
use futures::lock::Mutex;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use rand::prelude::*;
use std::sync::Arc;
use tokio::task::{spawn, JoinHandle};

#[tokio::main]
async fn main() {
    let scene_size = Coord { x: 54.0, y: 45.0 };
    let scale = 10;
    let (tx, rx): (Sender<(Coord, Arc<Effect>)>, Receiver<(Coord, Arc<Effect>)>) = unbounded();

    let scene = Arc::new(Scene::new(scene_size, scale));
    let dot_factory = DotFactory::new(tx.clone());

    for x in 0..scene_size.x as u16 {
        for y in 0..scene_size.y as u16 {
            let pos = Coord {
                x: x.into(),
                y: y.into(),
            };

            scene
                .push_dot(pos, dot_factory.create(pos, None, 1.0).await)
                .await;
            if x % 9 == 4 && y % 9 == 4 {
                for _ in 0..2 {
                    tx.send_async((
                        Coord {
                            x: x.into(),
                            y: y.into(),
                        },
                        Arc::new(Effect::SEED(Dna::new(thread_rng().gen::<[u64; SIZE]>()))),
                    ))
                    .await
                    .unwrap();
                }
            }
        }
    }

    let _ = spawn_propagator(rx.clone(), scene.clone());

    let open_gl_version = OpenGL::V3_2;
    let window: GlutinWindow = WindowSettings::new(
        "Dots",
        [scene_size.x * scale as f64, scene_size.y * scale as f64],
    )
    .exit_on_esc(true)
    .graphics_api(open_gl_version)
    .vsync(true)
    .build()
    .unwrap();
    let gl = GlGraphics::new(open_gl_version);
    let glyph_cache = Mutex::new(
        GlyphCache::new("assets/UbuntuMono-Regular.ttf", (), TextureSettings::new()).unwrap(),
    );

    let mut app = App::new(scene.clone(), window, gl, glyph_cache);
    let mut es = EventSettings::new();
    es.set_ups(12);
    es.set_max_fps(30);
    let mut events = Events::new(es);
    while let Some(e) = events.next(&mut app.window) {
        if let Some(r) = e.render_args() {
            app.render(&r).await;
        }
        if let Some(u) = e.update_args() {
            app.update(&u).await;
        }
    }
}

fn spawn_propagator(rx: Receiver<(Coord, Arc<Effect>)>, scene: Arc<Scene>) -> JoinHandle<()> {
    return spawn(async move {
        while let Ok((pos, effect)) = rx.recv_async().await {
            if let Some(dot) = scene.at(pos).await {
                let mut dot = dot.lock().await;
                dot.apply_effect(effect);
            }
        }
    });
}
