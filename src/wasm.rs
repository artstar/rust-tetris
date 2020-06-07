use wasm_bindgen::prelude::*;

use crate::bootstrap::{Action, Game, GameChange, GameMode, MenuMode, Settings, Timestamp};
use crate::game::tetris::Tetris;

mod bootstrap;

mod game {
    pub mod tetris;
}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct JSGame {
    settings: Settings,
    tetris: Tetris,
}

#[wasm_bindgen]
impl JSGame {
    pub fn start(time: Timestamp) -> JSGame {
        let settings = Settings {
            cols: 10,
            rows: 20,
            delay: 500,
        };
        let tetris = Tetris::new(settings, time);
        JSGame { settings, tetris }
    }

    pub fn tick(&mut self, time: Timestamp, action: Option<Action>) -> JSRender {
        let frame = self.tetris.frame(time, action);
        match frame {
            GameChange::Draw(gamemode) => JSRender {
                action: JSAction::Draw,
                gameview: Some(JSGame::gameview(&gamemode)),
                textview: None,
            },
            GameChange::Text(menumode) => JSRender {
                action: JSAction::Text,
                gameview: None,
                textview: Some(JSGame::textview(menumode)),
            },
            GameChange::Restart => {
                self.restart(time);
                JSRender {
                    action: JSAction::Idle,
                    gameview: None,
                    textview: None,
                }
            }
            GameChange::Exit => JSRender {
                action: JSAction::Exit,
                gameview: None,
                textview: None,
            },
            GameChange::Idle => JSRender {
                action: JSAction::Idle,
                gameview: None,
                textview: None,
            },
        }
    }

    fn restart(&mut self, time: Timestamp) {
        self.tetris = Tetris::new(self.settings, time);
    }

    fn gameview(gamemode: &GameMode) -> GameView {
        GameView {
            main: gamemode
                .main
                .iter()
                .flat_map(|row| row.to_owned())
                .collect(),
            preview: gamemode
                .preview
                .iter()
                .flat_map(|row| row.to_owned())
                .collect(),
            score: gamemode.score,
        }
    }

    fn textview<T>(menuview: &MenuMode<T>) -> TextView {
        TextView {
            items: menuview
                .get_items()
                .iter()
                .map(|item| item.string)
                .collect(),
            selected: *menuview.get_selected(),
        }
    }
}

pub struct GameView {
    main: Vec<u8>,
    preview: Vec<u8>,
    score: u32,
}

pub struct TextView {
    items: Vec<&'static str>,
    selected: Option<usize>,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum JSAction {
    Idle = 0,
    Exit = 1,
    Draw = 2,
    Text = 3,
}

#[wasm_bindgen]
pub struct JSRender {
    pub action: JSAction,
    gameview: Option<GameView>,
    textview: Option<TextView>,
}

#[wasm_bindgen]
impl JSRender {
    pub fn main(&self) -> Option<Vec<u8>> {
        Some(self.gameview.as_ref()?.main.clone())
    }

    pub fn preview(&self) -> Option<Vec<u8>> {
        Some(self.gameview.as_ref()?.preview.clone())
    }

    pub fn score(&self) -> Option<u32> {
        Some(self.gameview.as_ref()?.score)
    }

    pub fn text_items(&self) -> Option<String> {
        Some(self.textview.as_ref()?.items.join("\n"))
    }

    pub fn text_selected(&self) -> Option<u16> {
        Some(self.textview.as_ref()?.selected? as u16)
    }
}
