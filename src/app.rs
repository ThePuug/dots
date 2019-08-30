use glutin_window::{GlutinWindow};
use graphics::{clear,rectangle,text,Transformed};
use opengl_graphics::{GlGraphics,GlyphCache};
use piston::input::{RenderArgs,UpdateArgs};
use std::sync::{Arc,Mutex};
use std::time::{Instant};

use effects::{Effect,EffectType};
use neuralnets::{INeuralNet,INeuralNetFactory};
use physics::{Physics};

pub struct App<TNeuralNet,TNeuralNetFactory>
    where TNeuralNet : INeuralNet,
    TNeuralNetFactory : INeuralNetFactory<TNeuralNet> {
    pub gl: GlGraphics,
    pub window: GlutinWindow,
    pub physics: Physics<TNeuralNet,TNeuralNetFactory>,
    last_render: Instant,
    last_renders: Vec<u128>,
    last_render_sum: u128,
    glyph_cache_mutex: Mutex<GlyphCache<'static>>,
}

impl<TNeuralNet,TNeuralNetFactory> App<TNeuralNet,TNeuralNetFactory> 
    where TNeuralNet : INeuralNet, 
    TNeuralNetFactory : INeuralNetFactory<TNeuralNet> {
    pub fn new(physics: Physics<TNeuralNet,TNeuralNetFactory>,
               window: GlutinWindow,
               gl: GlGraphics,
               glyph_cache_mutex: Mutex<GlyphCache<'static>>) -> App<TNeuralNet,TNeuralNetFactory> {
        let last_renders: Vec<u128> = Vec::new();
        App {
            gl,
            window,
            physics,
            last_render: Instant::now(),
            last_renders,
            last_render_sum: 0,
            glyph_cache_mutex,
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        self.physics.apply();

        self.gl.draw(args.viewport(), |_c, gl| {            
            clear([1.0,1.0,1.0,1.0], gl); // Clear the screen.
        });

        for (_,mutex) in self.physics.scene.dots.lock().unwrap().iter() {
            let it = mutex.lock().unwrap();
            let dot = rectangle::centered_square(
                it.pos.x * self.physics.scene.scale as f64 + 0.5 * self.physics.scene.scale as f64, 
                it.pos.y * self.physics.scene.scale as f64 + 0.5 * self.physics.scene.scale as f64, 
                0.5 * self.physics.scene.scale as f64);
                self.gl.draw(args.viewport(), |c, gl| {            
                    rectangle(it.color, dot, c.transform, gl);
                });
        }

        let new = self.last_render.elapsed().as_millis();
        self.last_renders.push(new);
        self.last_render_sum += new;
        let frame_average_count = 100;
        let mut fps = 60; // assume 60 fps until we can generate the frame average
        if self.last_renders.len() > frame_average_count { 
            let old = self.last_renders.remove(0);
            self.last_render_sum -= old;
            fps = 1000 / (self.last_render_sum / frame_average_count as u128);
        }
        let glyph_cache = self.glyph_cache_mutex.get_mut().unwrap();
        self.gl.draw(args.viewport(), |c, gl| {
            text([1.0,0.0,0.0,1.0], 32, &format!("{}",fps), glyph_cache, c.transform.trans(10.0,42.0), gl).unwrap();
        });

        self.last_render = Instant::now();
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        self.physics.queue((Arc::new(Vec::new()),Arc::new(Effect {
            pos: None,
            typ: Some(EffectType::TICK),
            val: Some(1.0)
        })));
    }
}
