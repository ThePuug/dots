use futures::lock::{Mutex};
use glutin_window::{GlutinWindow};
use graphics::{clear,rectangle,text,Transformed};
use opengl_graphics::{GlGraphics,GlyphCache};
use piston::input::{RenderArgs,UpdateArgs};
use std::sync::{Arc};
use std::time::{Instant};

use crate::scene::{Scene};

pub struct App {
    pub gl: GlGraphics,
    pub window: GlutinWindow,
    scene: Arc<Scene>,
    last_render: Instant,
    renders: Vec<u128>,
    render_sum: u128,
    glyph_cache_mutex: Mutex<GlyphCache<'static>>,
}

impl App {
    pub fn new(scene: Arc<Scene>,
               window: GlutinWindow,
               gl: GlGraphics,
               glyph_cache_mutex: Mutex<GlyphCache<'static>>) -> App {
        let renders: Vec<u128> = Vec::new();
        App {
            scene,
            gl,
            window,
            last_render: Instant::now(),
            renders,
            render_sum: 0,
            glyph_cache_mutex,
        }
    }

    pub async fn render(&mut self, args: &RenderArgs) {
        // capture instants first for timings
        let millis_since = self.last_render.elapsed().as_millis();
        self.last_render = Instant::now();

        // Clear the screen.
        self.gl.draw(args.viewport(), |_c, gl| {            
            clear([0.0,0.0,0.0,1.0], gl);
        });

        // draw every dot
        for (x,y,sz,([r,g,b],opc)) in self.scene.describe().await {
            self.gl.draw(args.viewport(), |c, gl| {
                rectangle([r,g,b,opc], rectangle::centered_square(x,y,sz), c.transform, gl);
            });
        };

        // calculate fps as average of frame_average_count frames
        let frame_average_count = 30;
        self.renders.push(millis_since);
        self.render_sum += millis_since;
        let mut fps = 30; // assume 30 fps until we can generate the frame average
        if self.renders.len() > frame_average_count { 
            let least_recent = self.renders.remove(0);
            self.render_sum -= least_recent;
            fps = 1000 / (self.render_sum / frame_average_count as u128);
        }

        // render fps
        let glyph_cache = self.glyph_cache_mutex.get_mut();
        self.gl.draw(args.viewport(), |c, gl| {
            text([1.0,0.0,0.0,1.0], 32, &format!("{}",fps), glyph_cache, c.transform.trans(10.0,42.0), gl).unwrap();
        });
    }

    pub async fn update(&mut self, _args: &UpdateArgs) {}
}
