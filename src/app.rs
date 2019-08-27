use glutin_window::GlutinWindow;
use opengl_graphics::GlGraphics;
use piston::input::{RenderArgs,UpdateArgs};
use std::sync::Arc;

use effects::{Effect,EffectType};
use neuralnets::{INeuralNet,INeuralNetFactory};
use physics::Physics;

pub struct App<TNeuralNet,TNeuralNetFactory>
    where TNeuralNet : INeuralNet,
    TNeuralNetFactory : INeuralNetFactory<TNeuralNet> {
    pub gl: GlGraphics, // OpenGL drawing backend.
    pub window: GlutinWindow,
    pub physics: Physics<TNeuralNet,TNeuralNetFactory>
}

impl<TNeuralNet,TNeuralNetFactory> App<TNeuralNet,TNeuralNetFactory> 
    where TNeuralNet : INeuralNet, 
    TNeuralNetFactory : INeuralNetFactory<TNeuralNet> {
    pub fn new(physics: Physics<TNeuralNet,TNeuralNetFactory>,
               window: GlutinWindow,
               gl: GlGraphics) -> App<TNeuralNet,TNeuralNetFactory> {
        App {
            gl,
            window,
            physics
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |_c, gl| {
            clear([1.0,1.0,1.0,1.0], gl); // Clear the screen.
        });

        for (_,mutex) in self.physics.scene.dots.lock().unwrap().iter() {
            let it = mutex.lock().unwrap();
            let dot = rectangle::centered_square(
                it.pos.x * self.physics.scene.scale as f64, 
                it.pos.y * self.physics.scene.scale as f64, 
                0.5 * self.physics.scene.scale as f64);
            self.gl.draw(args.viewport(), |c, gl| {
                rectangle(it.color, dot, c.transform, gl); // draw it
            });
        }
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        self.physics.apply();
        self.physics.queue((Arc::new(Vec::new()),Arc::new(Effect {
            pos: None,
            typ: Some(EffectType::TICK),
            val: Some(1.0)
        })));
    }
}
