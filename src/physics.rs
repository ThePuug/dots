use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::mpsc::{Sender,Receiver};
use std::sync::Mutex;

use common::coord::Coord;
use common::threadpool::ThreadPool;
use dots::{IDot,IDotFactory};
use effects::{Effect,EffectType};
use intelligence::{IIntelligence};
use scene::Scene;

pub struct Physics<TDot,TDotFactory,TIntelligence> {
    pub scene: Scene<TDot>,
    rx: Receiver<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,
    tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,
    thread_pool: ThreadPool,
    dot_factory: TDotFactory,
    _t_intelligence: PhantomData<TIntelligence>,
}

impl<TDot,TDotFactory,TIntelligence> Physics<TDot,TDotFactory,TIntelligence>
    where TDot : IDot,
          TIntelligence : IIntelligence,
          TDotFactory : IDotFactory<TDot,TIntelligence> {
    pub fn new(thread_pool: ThreadPool,
               scene: Scene<TDot>,
               tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,
               rx: Receiver<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,
               dot_factory: TDotFactory) -> Physics<TDot,TDotFactory,TIntelligence> {
        let physics = Physics {
            scene,
            rx,
            tx,
            thread_pool,
            dot_factory,
            _t_intelligence: PhantomData,
        };

        // create the origin of life
        physics.queue((Arc::new(Vec::new()),Arc::new(Effect {
            pos: Some(Coord{x: 10.0, y: 10.0}),
            typ: Some(EffectType::OPACITY),
            val: Some(1.0)
        })));

        physics
    }

    pub fn queue(&self, msg: (Arc<Vec<Arc<Effect>>>,Arc<Effect>)) {
        self.tx.send(msg).unwrap();
    }

    pub fn apply(&self) {
        // drain the physics queue
        let mut rx_iter = self.rx.try_iter();
        while let Some((causes,effect)) = rx_iter.next() {

            match effect.pos {
                // when the pos is not defined, then send the effect to the scene for propagation
                None => {
                    self.scene.apply((causes,effect));
                }

                // if the pos is defined ensure the dot exists and create it if it does not
                Some(pos) => {
                    let dot: Arc<Mutex<TDot>>;
                    match self.scene.at(pos) {
                        Some(d) => {
                            dot = d;
                        },
                        None => {
                            dot = self.scene.push_dot(self.dot_factory.create(pos,[0.0,0.0,0.0,0.0]));
                        }
                    }

                    match effect.typ {
                        // when there is an effect, we want to update the dot
                        Some(_) => {
                            self.thread_pool.run(move || {
                                let mut dot = dot.lock().unwrap();
                                dot.apply_effect((causes,effect));
                            });
                        },

                        // when there is no effect, then the dot is sensing
                        None => {

                            // so populate the vals of the causes
                            let mut ret_causes: Vec<Arc<Effect>> = Vec::new();
                            for cause in causes.iter() {
                                match self.scene.at(cause.pos.unwrap()) { // TODO: will panic if pos is None

                                    // sense attributes of dots that exist
                                    Some(dot) => {
                                        ret_causes.push(Arc::new(Effect { 
                                            pos: cause.pos, 
                                            typ: cause.typ, 
                                            val: dot.lock().unwrap().describe(cause.typ.unwrap()) // TODO: will panic if typ is None
                                        }));
                                    },

                                    // return None val when dot does not exist
                                    None => {
                                        ret_causes.push(Arc::new(Effect { 
                                            pos: cause.pos, 
                                            typ: cause.typ,
                                            val: None
                                        }));
                                    }
                                }
                            }

                            // once causes are populated, allow the dot to respond
                            self.thread_pool.run(move || {
                                let mut dot = dot.lock().unwrap();
                                dot.act(Arc::new(ret_causes));
                            });
                        }
                    }
                }
            }
        }
    }
}