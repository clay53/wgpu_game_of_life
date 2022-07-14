use std::sync::{Arc, Mutex};

use bui::ttf::CachedFace;
use bui_basic::containers::Init;
use bui_basic::signal::{SignalReciever, ResizedSignal, RedrawCallback, CursorMovedSignal, MouseLeftDownSignal, MouseLeftUpSignal, CharacterInputSignal};
use winit::window::Window;

use crate::ResumeCallback;
use crate::setup::{Setup, SetupLeftUp};
use crate::game::{Game, GameSetupCallback};

pub enum Scene {
    Setup(Setup),
    Game(Game),
}

pub struct BuiView {
    scene: Scene,
    face: Arc<Mutex<CachedFace>>,
    resx: f32,
    resy: f32,
}

impl BuiView {
    pub fn new(face: Arc<Mutex<CachedFace>>, resx: f32, resy: f32) -> Self {
        Self {
            scene: Scene::Setup(Setup::new(face.clone(), resx, resy)),
            face,
            resx,
            resy,
        }
    }

    pub fn resume(&mut self, window: &Window) {
        match &mut self.scene {
            Scene::Setup(setup) => setup.resume(window),
            Scene::Game(game) => game.resume(window),
        }
    }

    pub fn render(&mut self) {
        match &mut self.scene {
            Scene::Setup(setup) => setup.render(),
            Scene::Game(game) => game.render(),
        }
    }
}

impl Init for BuiView {
    fn init(&mut self) {
        match &mut self.scene {
            Scene::Setup(setup) => setup.init(),
            Scene::Game(_game) => unreachable!(),
        }
    }
}

impl SignalReciever<ResizedSignal, RedrawCallback> for BuiView {
    fn take_signal(&mut self, signal: &mut ResizedSignal) -> RedrawCallback {
        self.resx = signal.resx;
        self.resy = signal.resy;
        match &mut self.scene {
            Scene::Setup(setup) => setup.take_signal(signal),
            Scene::Game(game) => game.take_signal(signal),
        }
    }
}

impl SignalReciever<CursorMovedSignal, RedrawCallback> for BuiView {
    fn take_signal(&mut self, signal: &mut CursorMovedSignal) -> RedrawCallback {
        match &mut self.scene {
            Scene::Setup(setup) => setup.take_signal(signal),
            Scene::Game(game) => {game.take_signal(signal); RedrawCallback::new(false)},
        }
    }
}

impl SignalReciever<MouseLeftDownSignal, RedrawCallback> for BuiView {
    fn take_signal(&mut self, signal: &mut MouseLeftDownSignal) -> RedrawCallback {
        match &mut self.scene {
            Scene::Setup(setup) => setup.take_signal(signal),
            Scene::Game(game) => {game.take_signal(signal); RedrawCallback::new(false)}
        }
    }
}

impl SignalReciever<MouseLeftUpSignal, (RedrawCallback, ResumeCallback)> for BuiView {
    fn take_signal(&mut self, signal: &mut MouseLeftUpSignal) -> (RedrawCallback, ResumeCallback) {
        match &mut self.scene {
            Scene::Setup(setup) => {
                match setup.take_signal(signal) {
                    SetupLeftUp::DoNothing => (RedrawCallback::new(false), ResumeCallback::new(false)),
                    SetupLeftUp::Go => {
                        match setup.get_dimensions() {
                            Ok(dimensions) => {
                                // TODO: Support sending renderer as a gift <3
                                let mut game = Game::new(self.face.clone(), self.resx, self.resy, dimensions.0, dimensions.1);
                                game.init();
                                self.scene = Scene::Game(game);
                                (RedrawCallback::new(true), ResumeCallback::new(true))
                            },
                            Err(err) => {
                                println!("{}", err);
                                let err = err.to_string();
                                setup.set_error(err);
                                setup.construct();
                                (RedrawCallback::new(true), ResumeCallback::new(false))
                            }
                        }
                    }
                }
            },
            Scene::Game(game) => {
                let (game_setup_callback, redraw_callback) = game.take_signal(signal);
                match game_setup_callback {
                    GameSetupCallback::Setup => {
                        let mut setup = Setup::new(self.face.clone(), self.resx, self.resy);
                        setup.init();
                        self.scene = Scene::Setup(setup);
                        (RedrawCallback::new(true), ResumeCallback::new(true))
                    },
                    GameSetupCallback::None => {
                        (redraw_callback, ResumeCallback::new(false))
                    }
                }
            }
        }
    }
}

impl SignalReciever<CharacterInputSignal, RedrawCallback> for BuiView {
    fn take_signal(&mut self, signal: &mut CharacterInputSignal) -> RedrawCallback {
        match &mut self.scene {
            Scene::Setup(setup) => setup.take_signal(signal),
            Scene::Game(_game) => RedrawCallback::new(false),
        }
    }
}