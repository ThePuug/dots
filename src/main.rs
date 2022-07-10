mod action;
mod app;
mod common;
mod dots;
mod effects;
mod scene;

use crate::app::{App};
use crate::common::coord::{Coord};
use crate::dots::{DotFactory};
use crate::effects::{Effect,EffectType};
use crate::scene::{Scene};

use futures::lock::{Mutex};
use glutin_window::{GlutinWindow};
use opengl_graphics::{GlGraphics,GlyphCache,OpenGL,TextureSettings};
use piston::event_loop::*;
use piston::input::*;
use piston::window::{WindowSettings};
use rand::prelude::*;
use std::sync::{Arc};
use flume::{unbounded,Sender,Receiver};
use tokio::task::{spawn,JoinHandle};

#[tokio::main]
async fn main() {

    let scene_size = Coord{x: 60.0, y: 60.0}; 
    let scale = 10;

    let (tx,rx): (Sender<Arc<Effect>>,Receiver<Arc<Effect>>) = unbounded();
    for n in 0..4 {
        let seq:[u8;8] = thread_rng().gen();
        for _ in 0..10 {
            tx.send_async(Arc::new(Effect {
                pos: Some(Coord{x: (scene_size.x*(f64::from(n/2)+1.0)/3.0).floor(), y: (scene_size.y/2.0).floor()+(if n%2==0 {1.0} else {-1.0})}),
                typ: Some(EffectType::OPACITY),
                val: Some(seq)
            })).await.unwrap();
        }
    }

    let scene = Arc::new(Scene::new(scene_size,scale));
    let _ = spawn_propagator(tx.clone(), rx.clone(), scene.clone());

    let open_gl_version = OpenGL::V3_2;
    let window: GlutinWindow = WindowSettings::new("Dots", [scene_size.x * scale as f64, scene_size.y * scale as f64])
        .exit_on_esc(true)
        .graphics_api(open_gl_version)
        .vsync(true)
        .build()
        .unwrap();
    let gl = GlGraphics::new(open_gl_version);
    let glyph_cache = Mutex::new(GlyphCache::new("assets/UbuntuMono-Regular.ttf", (), TextureSettings::new()).unwrap());

    let mut app = App::new(scene.clone(), window, gl, glyph_cache);
    let mut es = EventSettings::new();
    es.set_ups(12);
    es.set_max_fps(30);
    let mut events = Events::new(es);
    while let Some(e) = events.next(&mut app.window) {
        if let Some(r) = e.render_args() { app.render(&r).await; }
        if let Some(u) = e.update_args() { app.update(&u).await; }
    }
}

fn spawn_propagator(tx: Sender<Arc<Effect>>, rx: Receiver<Arc<Effect>>, scene: Arc<Scene>) -> JoinHandle<()> {
    let dot_factory = DotFactory::new(tx.clone());
    return spawn(async move {
        while let Ok(effect) = rx.recv_async().await {

            let pos = effect.pos.unwrap();
            if scene.at(pos).await.is_none() {
                let dot = Some(dot_factory.create(pos,effect.val.unwrap(),0.0).await);
                scene.push_dot(pos,dot.unwrap().clone()).await;
            }

            if let Some(dot) = scene.at(pos).await {
                let mut dot = dot.lock().await;
                dot.apply_effect(effect);
            }
        }
    });
}
