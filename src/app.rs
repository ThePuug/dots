use glutin_window::{GlutinWindow};
use graphics::{clear,rectangle,text,Transformed};
use opengl_graphics::{GlGraphics,GlyphCache};
use piston::input::{RenderArgs,UpdateArgs};
use std::sync::{Arc,Mutex};
use std::time::{Instant};

use dots::{IDot,IDotFactory};
use effects::{Effect,EffectType};
use intelligence::{IIntelligence};
use physics::{Physics};

pub struct App<TDot,TDotFactory,TIntelligence> {
    pub gl: GlGraphics,
    pub window: GlutinWindow,
    pub physics: Physics<TDot,TDotFactory,TIntelligence>,
    last_render: Instant,
    renders: Vec<u128>,
    render_sum: u128,
    glyph_cache_mutex: Mutex<GlyphCache<'static>>,
}

impl<TDot,TDotFactory,TIntelligence> App<TDot,TDotFactory,TIntelligence> 
    where TDot : IDot,
          TIntelligence : IIntelligence,
          TDotFactory : IDotFactory<TDot,TIntelligence> {
    pub fn new(physics: Physics<TDot,TDotFactory,TIntelligence>,
               window: GlutinWindow,
               gl: GlGraphics,
               glyph_cache_mutex: Mutex<GlyphCache<'static>>) -> App<TDot,TDotFactory,TIntelligence> {
        let renders: Vec<u128> = Vec::new(); // TODO: make renders vec an arg with specified capacity?
        App {
            gl,
            window,
            physics,
            last_render: Instant::now(),
            renders,
            render_sum: 0,
            glyph_cache_mutex,
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        // capture instants first for timings
        let millis_since = self.last_render.elapsed().as_millis();
        self.last_render = Instant::now();

        // catch up on physics events
        self.physics.apply();

        // Clear the screen.
        self.gl.draw(args.viewport(), |_c, gl| {            
            clear([1.0,1.0,1.0,1.0], gl);
        });

        // draw every dot
        for (x,y,sz,opc) in self.physics.scene.describe() {
            self.gl.draw(args.viewport(), |c, gl| {
                rectangle([0.0,0.0,0.0,opc], rectangle::centered_square(x,y,sz), c.transform, gl); // BUG: will panic when attribute is None
            });
        };

        // calculate fps as average of frame_average_count frames
        let frame_average_count = 100;
        self.renders.push(millis_since);
        self.render_sum += millis_since;
        let mut fps = 60; // assume 60 fps until we can generate the frame average
        if self.renders.len() > frame_average_count { 
            let least_recent = self.renders.remove(0);
            self.render_sum -= least_recent;
            fps = 1000 / (self.render_sum / frame_average_count as u128);
        }

        // TODO: figure out a way of rendering ui to generically avoid flickering values
        //       we think 1 second is a good balance between responsiveness and readability
        // render fps
        let glyph_cache = self.glyph_cache_mutex.get_mut().unwrap();
        self.gl.draw(args.viewport(), |c, gl| {
            text([1.0,0.0,0.0,1.0], 32, &format!("{}",fps), glyph_cache, c.transform.trans(10.0,42.0), gl).unwrap();
        });
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        self.physics.queue((Arc::new(Vec::new()),Arc::new(Effect {
            pos: None,
            typ: Some(EffectType::TICK),
            val: Some(1.0)
        })));
    }
}
