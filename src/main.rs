use std::collections::HashSet;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;

use rand::Rng;
use console::Term;
use figlet_rs::FIGfont;

const SYMBOL: char = '●';
const GOAL: char = '▓';
const ESC: &str = "\x1b";
const BOARD_SIZE: usize = 25;
const GEN_SIZE: usize = 13;

const BORDER: char = '░';
const WALL: char = '░';

enum Movement {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Copy)]
struct BoardCell {
    wall_right: bool,
    wall_bottom: bool,
}

#[derive(Debug, Clone)]
struct GameState {
    board: [[char; BOARD_SIZE]; BOARD_SIZE],
    position: Position,
    win_position: Position,
    victory: bool,
}

impl GameState {

    /// new
    pub fn new() -> GameState {
        let mut state = GameState {
            board: generate_maze(),
            position: Position {
                x: 0,
                y: 0,
            },
            win_position: Position { x: BOARD_SIZE-1, y: BOARD_SIZE-1 },
            victory: false,

        };
        state.board[state.position.y][state.position.x] = SYMBOL;
        state.board[state.win_position.y][state.win_position.x] = GOAL;

        return state;
    }

    /// move_position
    pub fn move_position(&mut self, action: Movement) {
        let mut new_pos = Position {
            y: self.position.y,
            x: self.position.x,
        };

        match action {
            Movement::UP => if let Some(y) = self.position.y.checked_sub(1) {
                new_pos.y = y;
            }
            Movement::DOWN => if self.position.y < BOARD_SIZE-1 {
                new_pos.y = self.position.y+1;
            }
            Movement::LEFT => if let Some(x) = self.position.x.checked_sub(1) {
                new_pos.x = x;
            }
            Movement::RIGHT => if self.position.x < BOARD_SIZE-1 {
                new_pos.x = self.position.x+1;
            }
        }

        if self.is_valid_move(&new_pos) {
            self.board[self.position.y][self.position.x] = ' ';
            self.board[new_pos.y][new_pos.x] = SYMBOL;
            self.position = new_pos;

            if self.is_win_position() {
                self.victory = true;
            }
        }
    }

    /// is_valid_move
    /// Accepts a board reference and the destination position.
    /// Returns true if move is valid, otherwise false.
    fn is_valid_move(&self, new_position: &Position) -> bool {
        !self.victory && self.board[new_position.y][new_position.x] != WALL
    }

    fn is_win_position(&self) -> bool {
        self.position == self.win_position
    }
}

/// generate_maze
fn generate_maze() -> [[char; BOARD_SIZE]; BOARD_SIZE] {
    let mut board = [[BoardCell{
        wall_right: false,
        wall_bottom: false,
    }; GEN_SIZE]; GEN_SIZE];
    let mut pos = Position{ y: GEN_SIZE/2, x: GEN_SIZE/2 };
    let mut visited = HashSet::new();
    let mut stack = vec![pos];

    let mut popped = false;
    while stack.len() > 0 && visited.len() < BOARD_SIZE.pow(2) {
        visited.insert(pos);

        let mut moves = vec![];
        // Move Up
        if pos.y > 0 {
            let mv = Position{ y: pos.y-1, x: pos.x };
            if !visited.contains(&mv) {
                moves.push(mv);
            } else if !popped && stack[stack.len()-1] != mv {
                board[mv.y][mv.x].wall_bottom = true;
            }
        }
        // Move Left
        if pos.x > 0 {
            let mv = Position{ y: pos.y, x: pos.x-1 };
            if !visited.contains(&mv){
                moves.push(mv);
            } else if !popped && stack[stack.len()-1] != mv {
                board[mv.y][mv.x].wall_right = true;
            }
        }
        // Move Down
        if pos.y < GEN_SIZE-1 {
            let mv = Position{ y: pos.y+1, x: pos.x };
            if !visited.contains(&mv) {
                moves.push(mv);
            } else if !popped && stack[stack.len()-1] != mv {
                board[pos.y][pos.x].wall_bottom = true;
            }
        }
        // Move Right
        if pos.x < GEN_SIZE-1 {
            let mv = Position{ y: pos.y, x: pos.x+1 };
            if !visited.contains(&mv) {
                moves.push(mv);
            } else if !popped && stack[stack.len()-1] != mv {
                board[pos.y][pos.x].wall_right = true;
            }
        }

        if moves.len() > 0 {
            stack.push(pos);
            popped = false;
            // Choose randomly where to move.
            let mut rng = rand::thread_rng();
            let move_idx = rng.gen_range(0..moves.len());
            pos = moves[move_idx];

        } else {
            pos = stack.pop().unwrap();
            popped = true;
        }
    }

    // Convert Board into render board.
    let mut render_board = [[' '; BOARD_SIZE]; BOARD_SIZE];

    for y in 0..GEN_SIZE {
        for x in 0..GEN_SIZE {
            let c = board[y][x];

            let ry = 2*y;
            let rx = 2*x;

            if y < GEN_SIZE-1 {
                if x < GEN_SIZE-1 {
                    if c.wall_bottom {
                        render_board[ry+1][rx] = WALL;
                    }
                    if c.wall_right {
                        render_board[ry][rx+1] = WALL;
                    }
                    render_board[ry+1][rx+1] = WALL;
                } else {
                    if c.wall_bottom {
                        render_board[ry+1][rx] = WALL;
                    }
                }

            } else {
                if c.wall_right {
                    render_board[ry][rx+1] = WALL;
                }
            }
        }
    }

    render_board
}

/// main function
fn main() {
    let mut state = GameState::new();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || render(rx));
    if let Err(e) = tx.send(state.clone()) {
        panic!("Could not send board state to render {e}");
    }

    let stdout = Term::buffered_stdout();

    loop {
        if let Ok(c) = stdout.read_char() {
            match c {
                'w' => state.move_position(Movement::UP),
                'a' => state.move_position(Movement::LEFT),
                's' => state.move_position(Movement::DOWN),
                'd' => state.move_position(Movement::RIGHT),
                _ => (),
            };

            // Render
            if let Err(e) = tx.send(state.clone()) {
                panic!("Could not send board state to render {e}");
            }
        }
    }
}

/// render
fn render(rx: mpsc::Receiver<GameState>) {
    let (wd, ht) = term_size::dimensions().unwrap_or((BOARD_SIZE + 1, BOARD_SIZE + 1));
    let (draw_x, draw_y) = ((wd - (3 * BOARD_SIZE + 1)) / 2, (ht - BOARD_SIZE + 1) / 2);

    loop {
        if let Ok(state) = rx.recv() {
            print!("{ESC}[2J{ESC}[{draw_y};{draw_x}H");

            // Draw Top Border
            for _ in 0..(3 * BOARD_SIZE + 2) {
                print!("{BORDER}");
            }

            print!("{ESC}[E{ESC}[{draw_x}G");
            // Draw each row
            for row in state.board.iter() {
                // Left Border
                print!("{BORDER}");
                for v in row.iter() {
                    match *v {
                        SYMBOL => print!("◀◆▶"),
                        GOAL => print!("{ESC}[35m{v}{v}{v}{ESC}[0m"),
                        _ => print!("{v}{v}{v}"),
                    }
                }
                // Right Border
                print!("{BORDER}\n");
                print!("{ESC}[{draw_x}G");
            }

            // Draw Bottom Border
            for _ in 0..(3 * BOARD_SIZE + 2) {
                print!("{BORDER}");
            }

            if state.victory {
                let ffont = FIGfont::standand().unwrap();
                if let Some(msg) = ffont.convert("You Did It!") {
                    let mut m_w = msg.to_string().lines().map(|s| s.len()).max().unwrap_or(1);
                    let mut m_h = msg.height as usize;
                    if m_w > wd || m_h > ht {
                        m_w = 0;
                        m_h = 0;
                    }
                    let midpoint = ((wd - m_w) / 2, (ht - m_h) / 2);
                    for (i, l) in msg.to_string().lines().enumerate() {
                        print!("{ESC}[{ht};{w}H", w = midpoint.0, ht = midpoint.1 + i);
                        print!("{}", l);
                    }
                }
            }

            io::stdout().flush().unwrap();
        }
    }
}
