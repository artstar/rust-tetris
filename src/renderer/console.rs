use crate::bootstrap::{Action, GameMode, MenuMode, Renderable, Settings};
use crate::renderer::console::ConsoleSymbol::{Simple, Styled};
use crossterm::event::{read, Event, KeyCode};
use crossterm::style::{Color, ContentStyle, Print, PrintStyledContent, StyledContent};
use crossterm::{cursor, terminal, Command, ExecutableCommand, QueueableCommand};
use std::cell::RefCell;
use std::fmt::{self, Display};
use std::io::{self, stdout, Stdout, Write};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

pub struct ConsoleView {
    settings: Settings,
    stdout: RefCell<Stdout>,
    width: u16,
    height: u16,
    char: char,
    color: Option<Color>,
}

impl ConsoleView {
    pub fn new(
        settings: Settings,
        width: u16,
        height: u16,
        char: char,
        color: Option<Color>,
    ) -> ConsoleView {
        let stdout = RefCell::new(stdout());
        ConsoleView {
            settings,
            stdout,
            width,
            height,
            char,
            color,
        }
    }

    pub fn prepare(&self) -> Result<()> {
        self.stdout.borrow_mut().execute(cursor::Hide)?;
        terminal::enable_raw_mode()?;
        self.clear()
    }

    pub fn clear(&self) -> Result<()> {
        self.stdout
            .borrow_mut()
            .execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    pub fn print_cell(&self, x: u16, y: u16, filled: bool, flush: bool) -> Result<()> {
        let symbol = self.styled(if filled { self.char } else { ' ' });
        for i in 0..self.width {
            for j in 0..self.height {
                self.print_styled(x * self.width + i + 1, y * self.height + j + 1, &symbol)?
            }
        }
        if flush {
            self.stdout.borrow_mut().flush()?;
        }
        Ok(())
    }

    /// Prints border for square area like this
    ///  +----+
    ///  |    |
    ///  |    |
    ///  +----+
    pub fn print_border(&self, left: u16, top: u16, cols: u16, rows: u16) -> Result<()> {
        let ceil = self.styled('-');
        let wall = self.styled('|');
        let corner = self.styled('+');
        for x in left..left + cols {
            for i in 0..self.width {
                self.print_styled(x * self.width + i + 1, top * self.height, &ceil)?;
                self.print_styled(
                    x * self.width + i + 1,
                    (top + rows) * self.height + 1,
                    &ceil,
                )?;
            }
        }
        for y in top..top + rows {
            for j in 0..self.height {
                self.print_styled(left * self.width, y * self.height + j + 1, &wall)?;
                self.print_styled(
                    (left + cols) * self.width + 1,
                    y * self.height + j + 1,
                    &wall,
                )?;
            }
        }
        self.print_styled(left * self.width, top * self.height, &corner)?;
        self.print_styled(left * self.width, (top + rows) * self.height + 1, &corner)?;
        self.print_styled((left + cols) * self.width + 1, top * self.height, &corner)?;
        self.print_styled(
            (left + cols) * self.width + 1,
            (top + rows) * self.height + 1,
            &corner,
        )?;
        self.stdout.borrow_mut().flush()?;
        Ok(())
    }

    fn empty(&self) -> Result<()> {
        for y in 0..self.settings.rows {
            for x in 0..self.settings.cols {
                self.print_cell(x as u16, y as u16, false, false)?;
            }
        }
        Ok(())
    }

    pub fn print_all(&self, frame: &Renderable) -> Result<()> {
        for (y, row) in frame.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if (x as u16) < self.settings.cols && (y as u16) < self.settings.rows {
                    self.print_cell(x as u16, y as u16, *cell > 0u8, false)?;
                }
            }
        }
        self.stdout.borrow_mut().flush()?;
        Ok(())
    }

    pub fn print_preview(&self, preview: &Renderable) -> Result<()> {
        for (y, row) in preview.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if (x as u16) < self.settings.cols && (y as u16) < self.settings.rows {
                    self.print_cell(
                        self.settings.cols + 2 + x as u16,
                        y as u16,
                        *cell > 0u8,
                        false,
                    )?;
                }
            }
        }
        self.stdout.borrow_mut().flush()?;
        Ok(())
    }

    pub fn print_score(&self, score: u32) -> Result<()> {
        let left = (self.settings.cols + 2) * self.width;
        let top = 6 * self.height;
        self.stdout
            .borrow_mut()
            .execute(cursor::MoveTo(left, top))?;
        write!(self.stdout.borrow_mut(), "{}", score)?;
        self.stdout.borrow_mut().flush()?;
        Ok(())
    }

    pub fn draw_game(&self, game: &GameMode) -> Result<()> {
        self.print_all(&game.main)?;
        self.print_preview(&game.preview)?;
        self.print_score(game.score)?;
        Ok(())
    }

    pub fn draw_text<T>(&self, menu: &MenuMode<T>) -> Result<()> {
        self.empty()?;
        for (idx, item) in menu.get_items().iter().enumerate() {
            let out = if matches!(menu.get_selected(), Some(x) if *x == idx) {
                String::from("-> ") + item.string + " <-"
            } else {
                item.string.to_string()
            };
            let left = self.settings.cols * self.width / 2 - (1 + out.len() as u16) / 2;
            self.stdout
                .borrow_mut()
                .execute(cursor::MoveTo(1 + left, idx as u16 * 2 + 1))?;
            write!(self.stdout.borrow_mut(), "{}", out)?;
        }
        self.stdout.borrow_mut().flush()?;
        Ok(())
    }

    pub fn init_field(&self) -> Result<()> {
        // Main gamefield
        self.print_border(0, 0, self.settings.cols, self.settings.rows)?;
        // Preview field
        self.print_border(self.settings.cols + 2, 0, 4, 4)?;
        Ok(())
    }

    pub fn keypress(&self, tx: Sender<Action>) -> JoinHandle<Result<()>> {
        thread::spawn(move || loop {
            let action = match read()? {
                Event::Key(event) => ConsoleView::key_to_action(event.code),
                _ => None,
            };
            if let Some(action) = action {
                tx.send(action)?;
            };
        })
    }

    fn print<A: Display>(&self, x: u16, y: u16, symbol: impl Command<AnsiType = A>) -> Result<()> {
        self.stdout
            .borrow_mut()
            .queue(cursor::MoveTo(x, y))?
            .queue(&symbol)?;
        Ok(())
    }

    fn styled(&self, symbol: char) -> ConsoleSymbol<char> {
        match self.color {
            Some(color) => ConsoleSymbol::Styled(PrintStyledContent(StyledContent::new(
                ContentStyle::new().foreground(color),
                symbol,
            ))),
            None => ConsoleSymbol::Simple(Print(symbol)),
        }
    }

    fn print_styled(&self, x: u16, y: u16, symbol: &ConsoleSymbol<char>) -> Result<()> {
        match &symbol {
            Styled(s) => self.print(x, y, &s),
            Simple(s) => self.print(x, y, &s),
        }
    }

    fn key_to_action(key: KeyCode) -> Option<Action> {
        match key {
            KeyCode::Up | KeyCode::Char('w') => Some(Action::Up),
            KeyCode::Down | KeyCode::Char('s') => Some(Action::Down),
            KeyCode::Left | KeyCode::Char('a') => Some(Action::Left),
            KeyCode::Right | KeyCode::Char('d') => Some(Action::Right),
            KeyCode::Enter | KeyCode::Char(' ') => Some(Action::Drop),
            KeyCode::Esc | KeyCode::Backspace => Some(Action::Escape),
            _ => None,
        }
    }
}

pub enum ConsoleSymbol<D: Display + Clone> {
    Styled(PrintStyledContent<D>),
    Simple(Print<D>),
}

type Result<T> = std::result::Result<T, ConsoleViewError>;

#[derive(Debug)]
pub enum ConsoleViewError {
    ConsoleErr(crossterm::ErrorKind),
    IOErr(io::Error),
    SendError(mpsc::SendError<Action>),
}

impl Display for ConsoleViewError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConsoleViewError::ConsoleErr(ref e) => e.fmt(f),
            ConsoleViewError::IOErr(ref e) => e.fmt(f),
            ConsoleViewError::SendError(ref e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ConsoleViewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ConsoleViewError::ConsoleErr(ref e) => Some(e),
            ConsoleViewError::IOErr(ref e) => Some(e),
            ConsoleViewError::SendError(ref e) => Some(e),
        }
    }
}

impl From<crossterm::ErrorKind> for ConsoleViewError {
    fn from(err: crossterm::ErrorKind) -> ConsoleViewError {
        ConsoleViewError::ConsoleErr(err)
    }
}

impl From<io::Error> for ConsoleViewError {
    fn from(err: io::Error) -> ConsoleViewError {
        ConsoleViewError::IOErr(err)
    }
}

impl From<mpsc::SendError<Action>> for ConsoleViewError {
    fn from(err: mpsc::SendError<Action>) -> ConsoleViewError {
        ConsoleViewError::SendError(err)
    }
}
