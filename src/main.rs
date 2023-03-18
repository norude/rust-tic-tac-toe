use std::{fmt, io::Write};

use termion::{cursor, event::Key, input::TermRead, raw::IntoRawMode};

#[derive(Debug, Copy, Clone, Default, PartialEq)]
enum State {
    #[default]
    None,
    Cross,
    Nought,
}
use State::{Cross, Nought};
#[derive(Debug, Copy, Clone, Default)]
struct Cell {
    state: State,
    highlighted: bool,
}
#[derive(Debug, Copy, Clone, Default)]
struct Board {
    grid: [[Cell; 3]; 3],
    highlighted: Option<(usize, usize)>,
}
enum Dir {
    Left,
    Right,
    Up,
    Down,
}
impl Board {
    fn modify(&mut self, x: usize, y: usize, set: State) {
        self.grid[x][y].state = set
    }
    fn highlight(&mut self, x: usize, y: usize) {
        self.grid[x][y].highlighted = true;
        self.highlighted = Some((x, y));
    }
    fn move_highlight(&mut self, dir: Dir) -> Result<(), String> {
        self.highlighted
            .map_or(Err("no previous value".to_string()), |(mut x, mut y)| {
                {
                    self.grid[x][y].highlighted = false;
                    match dir {
                        Dir::Down => y = (y + 1) % 3,
                        Dir::Up => y = (y + 2) % 3,
                        Dir::Right => x = (x + 1) % 3,
                        Dir::Left => x = (x + 2) % 3,
                    }
                    self.highlight(x, y)
                }
                Ok(())
            })
    }
    fn do_move(&mut self, stdin: &mut termion::input::Keys<termion::AsyncReader>, state: State) {
        loop {
            match self.get_place(stdin) {
                Some(place) => match self.grid[place.0][place.1].state {
                    State::None => {
                        self.modify(place.0, place.1, state);
                        break;
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }
    }
    fn get_place(
        &mut self,
        stdin: &mut termion::input::Keys<termion::AsyncReader>,
    ) -> Option<(usize, usize)> {
        let mut stdout = std::io::stdout().into_raw_mode().ok()?;
        write!(stdout, "{}{}", self, cursor::Goto(1, 1)).ok()?;
        loop {
            if let Some(Ok(key)) = stdin.next() {
                self.move_highlight(match key {
                    Key::Char('w') => Dir::Up,
                    Key::Char('a') => Dir::Left,
                    Key::Char('s') => Dir::Down,
                    Key::Char('d') => Dir::Right,
                    Key::Char('\n') => {
                        return self.highlighted;
                    }
                    _ => continue,
                })
                .ok()?;
                write!(stdout, "{}{}", self, cursor::Goto(1, 1)).ok()?;
            }
        }
    }
    fn is_full(&self) -> bool {
        for x in self.grid {
            for y in x {
                if let State::None = y.state {
                    return false;
                }
            }
        }
        true
    }
    fn did_won(&self, state: State) -> bool {
        let n = self.grid.len();
        self.grid.iter().any(|i| i.iter().all(|j| j.state == state))
            || (0..n).any(|i| self.grid.iter().all(|j| j[i].state == state))
            || (0..n).all(|i| self.grid[i][i].state == state)
            || (0..n).all(|i| self.grid[i][n - i - 1].state == state)
    }
}
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Cross => write!(f, "x"),
            Nought => write!(f, "○"),
            State::None => write!(f, " "),
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.highlighted {
            write!(f, "\x1b[47m{}\x1b[0m", self.state)
        } else {
            write!(f, "{}", self.state)
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "┌───┬───┬───┐\n\r│ {} │ {} │ {} │\n\r├───┼───┼───┤\n\r│ {} │ {} │ {} │\n\r├───┼───┼───┤\n\r│ {} │ {} │ {} │\n\r└───┴───┴───┘\n\r",
            self.grid[0][0],
            self.grid[1][0],
            self.grid[2][0],
            self.grid[0][1],
            self.grid[1][1],
            self.grid[2][1],
            self.grid[0][2],
            self.grid[1][2],
            self.grid[2][2]
        )
    }
}

fn main() {
    let mut stdin = termion::async_stdin().keys();
    let mut grid: Board = Default::default();
    grid.highlight(1, 1);

    loop {
        for &player in &[Cross, Nought] {
            grid.do_move(&mut stdin, player);

            if grid.is_full() {
                println!("{}Draw!", termion::clear::All);
                return;
            }

            if grid.did_won(player) {
                println!("{}{:?} won!", termion::clear::All, player);
                return;
            }
        }
    }
}
