use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Renderable(pub Vec<Vec<u8>>);

impl Deref for Renderable {
    type Target = Vec<Vec<u8>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Renderable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Settings {
    // If cols, rows < 5 something will crash.
    pub cols: u16,
    pub rows: u16,
    pub delay: Duration,
}

#[derive(Debug)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Drop,
    Escape,
}

#[derive(Debug)]
pub struct GameMode {
    pub main: Renderable,
    pub preview: Renderable,
    pub score: u32,
}

#[derive(Debug)]
pub struct MenuItem<T> {
    pub id: T,
    pub string: String,
    pub top: u16,
    pub selectable: bool,
}

#[derive(Debug)]
pub struct MenuMode<T> {
    items: Vec<MenuItem<T>>,
    selected: Option<usize>,
}

impl<T> MenuMode<T> {
    pub fn new(items: Vec<MenuItem<T>>) -> MenuMode<T> {
        let first_selectable = items.iter().position(|item| item.selectable);
        MenuMode {
            items,
            selected: first_selectable,
        }
    }

    pub fn down(&mut self) {
        match self.selected {
            None => {}
            Some(selected) => {
                let next = self
                    .items
                    .iter()
                    .enumerate()
                    .find(|(i, item)| *i > selected && item.selectable);
                self.selected = if let Some((i, _)) = next {
                    Some(i)
                } else {
                    self.items.iter().position(|item| item.selectable)
                }
            }
        };
    }

    pub fn up(&mut self) {
        match self.selected {
            None => {}
            Some(selected) => {
                let next = self
                    .items
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(i, item)| *i < selected && item.selectable);
                self.selected = if let Some((i, _)) = next {
                    Some(i)
                } else {
                    self.items.iter().rposition(|item| item.selectable)
                }
            }
        };
    }

    pub fn select(&self) -> Option<&T> {
        match self.selected {
            None => None,
            Some(idx) => Some(&self.items[idx].id),
        }
    }

    pub fn get_items(&self) -> &Vec<MenuItem<T>> {
        &self.items
    }

    pub fn get_selected(&self) -> &Option<usize> {
        &self.selected
    }
}

#[derive(Debug)]
pub enum GameChange<'a, T> {
    Draw(GameMode),
    Text(&'a MenuMode<T>),
    Restart,
    Exit,
    Idle,
}
