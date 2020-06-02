pub mod tetrisfield;
pub mod tetromino;

use crate::game::tetris::tetrisfield::TetrisField;
use crate::game::tetris::tetromino::{Block, Tetromino, I, J, L, O, S, T, Z};
use crate::settings::{Action, GameChange, GameMode, MenuItem, MenuMode, Renderable, Settings};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::mem;
use std::time::Instant;

pub enum TetrisPause {
    Title,
    Continue,
    Restart,
    Exit,
}

pub struct Tetris<'a> {
    settings: &'a Settings,
    moment: Instant,
    field: TetrisField,
    state: GameState,
    pause: Option<MenuMode<TetrisPause>>,
    score: u32,
    bag: Vec<Tetromino>,
}

impl<'a> Tetris<'a> {
    pub fn new(settings: &'a Settings, start: Instant) -> Tetris {
        let field = TetrisField::new(&settings);
        Tetris {
            moment: start,
            settings,
            field,
            state: GameState::Start,
            pause: None,
            score: 0,
            bag: vec![],
        }
    }

    pub fn random_block(&mut self) -> Tetromino {
        if self.bag.is_empty() {
            for _ in 0..BAG_SIZE {
                self.bag.append(&mut vec![
                    Tetromino::from(I()),
                    Tetromino::from(T()),
                    Tetromino::from(J()),
                    Tetromino::from(L()),
                    Tetromino::from(S()),
                    Tetromino::from(Z()),
                    Tetromino::from(O()),
                ])
            }
            let mut rng = thread_rng();
            self.bag.shuffle(&mut rng);
        }
        self.bag.pop().unwrap()
    }

    pub fn frame(&mut self, now: Instant, action: Option<Action>) -> GameChange<TetrisPause> {
        match &mut self.pause {
            None => {
                if matches!(action, Some(Action::Escape)) {
                    self.pause = Some(Tetris::pause_menu());
                } else {
                    match &self.state {
                        GameState::Start => self.state_start(),
                        GameState::Fall(_, _) => {
                            if !self.state_fall(now, action) {
                                return GameChange::Idle;
                            }
                        }
                        GameState::Drop(_, _) => self.state_drop(),
                        GameState::GameOver => self.pause = Some(Tetris::over_menu()),
                        GameState::Temp => unreachable!(),
                    }
                }
            }
            Some(menu) => match action {
                Some(Action::Escape) => self.pause = None,
                Some(Action::Up) => menu.up(),
                Some(Action::Down) => menu.down(),
                Some(Action::Drop) => match menu.select() {
                    Some(TetrisPause::Continue) => self.pause = None,
                    Some(TetrisPause::Restart) => return GameChange::Restart,
                    Some(TetrisPause::Exit) => return GameChange::Exit,
                    _ => unreachable!(),
                },
                _ => return GameChange::Idle,
            },
        }
        match self.pause {
            Some(ref menu) => GameChange::Text(menu),
            None => GameChange::Draw(self.to_drawable()),
        }
    }

    pub fn state_start(&mut self) {
        let block = Block::spawn(self.random_block(), &self.settings);
        self.run_cicle(block);
    }

    pub fn state_fall(&mut self, now: Instant, action: Option<Action>) -> bool {
        let mut drop = false;
        let mut changed = false;
        if let GameState::Fall(ref mut block, _) = &mut self.state {
            match action {
                Some(Action::Left) => changed = self.field.try_move(block, -1, 0),
                Some(Action::Right) => changed = self.field.try_move(block, 1, 0),
                Some(Action::Down) => {
                    self.moment = now;
                    if self.field.try_move(block, 0, 1) {
                        changed = true
                    } else {
                        drop = true
                    }
                }
                Some(Action::Drop) => {
                    changed = self.field.drop(block) > 0;
                    drop = true;
                }
                Some(Action::Up) => changed = self.field.try_rotate(block),
                _ => changed = false,
            };

            if now.saturating_duration_since(self.moment) >= self.settings.delay {
                self.moment = now;
                if self.field.try_move(block, 0, 1) {
                    changed = true
                } else {
                    drop = true
                }
            }
        }
        if drop {
            if let GameState::Fall(block, next) = mem::take(&mut self.state) {
                self.state = GameState::Drop(block, next);
            }
        }
        changed
    }

    pub fn state_drop(&mut self) {
        if let GameState::Drop(prev, current) = mem::take(&mut self.state) {
            let lines = self.field.consume(prev);
            self.score += (lines * (lines + 1) / 2) as u32;
            let block = Block::spawn(current, &self.settings);
            self.run_cicle(block);
        }
    }

    pub fn run_cicle(&mut self, block: Block) {
        let next = self.random_block();
        if self.field.has_collision(&block) {
            self.state = GameState::GameOver;
        } else {
            self.state = GameState::Fall(block, next);
        }
    }

    pub fn pause_menu() -> MenuMode<TetrisPause> {
        MenuMode::new(vec![
            MenuItem {
                id: TetrisPause::Title,
                string: "Menu".into(),
                top: 1,
                selectable: false,
            },
            MenuItem {
                id: TetrisPause::Continue,
                string: "Continue".into(),
                top: 3,
                selectable: true,
            },
            MenuItem {
                id: TetrisPause::Restart,
                string: "New Game".into(),
                top: 5,
                selectable: true,
            },
            MenuItem {
                id: TetrisPause::Exit,
                string: "Exit".into(),
                top: 7,
                selectable: true,
            },
        ])
    }

    pub fn over_menu() -> MenuMode<TetrisPause> {
        MenuMode::new(vec![
            MenuItem {
                id: TetrisPause::Title,
                string: "You Died".into(),
                top: 1,
                selectable: false,
            },
            MenuItem {
                id: TetrisPause::Restart,
                string: "New Game".into(),
                top: 3,
                selectable: true,
            },
            MenuItem {
                id: TetrisPause::Exit,
                string: "Exit".into(),
                top: 5,
                selectable: true,
            },
        ])
    }

    pub fn to_drawable(&self) -> GameMode {
        match &self.state {
            GameState::Fall(block, next) | GameState::Drop(block, next) => GameMode {
                main: self.field.field_with_block(&block),
                preview: next.preview(),
                score: self.score,
            },
            GameState::Start | GameState::GameOver => GameMode {
                main: self.field.field(),
                preview: Renderable(vec![vec![]]),
                score: self.score,
            },
            GameState::Temp => unreachable!(),
        }
    }
}

pub enum GameState {
    Start,
    Fall(Block, Tetromino),
    Drop(Block, Tetromino),
    GameOver,
    Temp,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Temp
    }
}

const BAG_SIZE: u8 = 3;
