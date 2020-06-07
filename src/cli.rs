mod renderer {
    pub mod console;
}

mod game {
    pub mod tetris;
}

mod bootstrap;

use crate::bootstrap::{Action, Game, GameChange, Settings, Timestamp};
use crate::game::tetris::Tetris;
use crate::renderer::console::ConsoleView;
use log::LevelFilter;
use std::error::Error;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn Error>> {
    simple_logging::log_to_file("test.log", LevelFilter::Info)?;

    let settings = Settings {
        cols: 10,
        rows: 20,
        delay: 500,
    };
    let (tx, rx) = mpsc::channel::<Action>();
    let renderer = ConsoleView::new(settings, 2, 1, '\u{2588}', None);
    renderer.prepare()?;
    renderer.init_field()?;
    renderer.keypress(tx);

    let now = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as Timestamp
    };
    let mut tetris = Tetris::new(settings, now());
    loop {
        let action = rx.try_recv().ok();
        let frame = tetris.frame(now(), action);
        match frame {
            GameChange::Draw(gameview) => renderer.draw_game(&gameview)?,
            GameChange::Text(ref menuview) => renderer.draw_text(&menuview)?,
            GameChange::Restart => tetris = Tetris::new(settings, now()),
            GameChange::Exit => break,
            GameChange::Idle => continue,
        }
    }
    renderer.clear()?;
    Ok(())
}
