use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::{Instant, Duration};

fn main() {
    let completed_board_arc = Arc::new((Mutex::new(None), Condvar::new()));
    let completed_board_arc_cloned = completed_board_arc.clone();
    thread::spawn(move|| {
        let (mutex, cvar) = &*completed_board_arc_cloned;
        loop {
            let mut completed_board_lock = mutex.lock().unwrap();
            let lock = cvar.wait(completed_board_lock).unwrap();
            let BoardPrint {
                board,
                board_num,
                board_find_time,
            } = lock.clone().take().expect("condvar set without board");

            print!("\x1B[2J\x1B[1;1H");
            println!("complete board #{} of size {} found", board_num, board.side_size);
            board.print_board();
            println!("Press Ctrl+C to exit");

            if let Some(time) = board_find_time {
                println!("finding all valid boards of size {} took {:?}", board.side_size, time);
            }
        }
    });
    for side_size in 4.. {
        print!("\x1B[2J\x1B[1;1H");
        let base_board = Board::new(side_size);
        let mut num_boards = 0;
        let start_time = Instant::now();
        find_valid_boards(&base_board, 0, &mut num_boards, &completed_board_arc);
        let end_time = Instant::now();
        {
            let mut lock = completed_board_arc.0.lock().unwrap();
            lock.as_mut().unwrap().board_find_time = Some(end_time - start_time);
            completed_board_arc.1.notify_one();
        }
        thread::sleep_ms(2000);
    }
}

fn find_valid_boards(
    base_board: &Board,
    col: usize,
    num_boards: &mut usize,
    completed_board_arc: &Arc<(Mutex<Option<BoardPrint>>, Condvar)>,
) {
    if base_board.is_complete() {
        *num_boards += 1;
        if let Ok(mut lock) = completed_board_arc.0.try_lock() {
            *lock = Some(BoardPrint {
                board: base_board.clone(),
                board_num: *num_boards,
                board_find_time: None,
            });
            completed_board_arc.1.notify_one();
        }
        return;
    }

    for child_board in base_board.valid_direct_children_with_queen_in_col(col) {
        find_valid_boards(&child_board, col + 1, num_boards, completed_board_arc);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BoardPrint {
    board: Board,
    board_num: usize,
    board_find_time: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Queen {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Board {
    queens: Vec<Queen>,
    side_size: usize,
}

impl Board {
    fn new(side_size: usize) -> Board {
        Board {
            queens: vec![],
            side_size,
        }
    }

    fn print_board(&self) {
        let mut string = String::new();
        for y in 0..self.side_size {
            for x in 0..self.side_size {
                if self.queens.contains(&Queen::new(x, y)) {
                    string += "QQ";
                } else {
                    string += "__";
                }
            }
            string += "\n";
        }
        println!("{}", string);
    }

    fn is_complete(&self) -> bool {
        self.queens.len() == self.side_size
    }

    fn valid_direct_children_with_queen_in_col(&self, col: usize) -> impl '_ + Iterator<Item=Board> {
        (0..self.side_size)
            .map(move |row| Queen::new(col, row))
            .filter_map(move |queen| self.try_insert_queen(queen))
    }

    fn try_insert_queen(&self, queen: Queen) -> Option<Board> {
        assert!(queen.x < self.side_size);
        assert!(queen.y < self.side_size);

        for q in &self.queens {
            if *q == queen {
                return None;
            }
        }

        let mut new_board = self.clone();
        new_board.queens.push(queen);
        new_board.queens.sort();
        if new_board.is_valid() {
            Some(new_board)
        } else {
            None
        }
    }

    fn is_valid(&self) -> bool {
        let mut occupied_rows = [false; 32];
        let mut occupied_cols = [false; 32];
        let mut occupied_sw_diagonals = [false; 32];
        let mut occupied_se_diagonals = [false; 32];

        for q in &self.queens {
            let row = q.row();
            let col = q.col();
            let sw_diagonal = q.sw_diagonal(self.side_size);
            let se_diagonal = q.se_diagonal(self.side_size);

            if occupied_rows[row] {
                return false;
            } else {
                occupied_rows[row] = true;
            }
            if occupied_cols[col] {
                return false;
            } else {
                occupied_cols[col] = true;
            }
            if occupied_sw_diagonals[sw_diagonal] {
                return false;
            } else {
                occupied_sw_diagonals[sw_diagonal] = true;
            }
            if occupied_se_diagonals[se_diagonal] {
                return false;
            } else {
                occupied_se_diagonals[se_diagonal] = true;
            }
        }

        return true;
    }
}

impl Queen {
    fn new(x: usize, y: usize) -> Queen {
        Queen{ x, y }
    }
    fn row(&self) -> usize {
        self.y
    }

    fn col(&self) -> usize {
        self.x
    }

    fn sw_diagonal(&self, board_side_size: usize) -> usize {
        board_side_size + self.x - self.y - 1
    }

    fn se_diagonal(&self, board_side_size: usize) -> usize {
        self.x + self.y
        // board_side_size + self.y - self.x - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sw_diagonal() {
        let bs = 8;
        assert_eq!(Queen::new(0, 1).sw_diagonal(bs), 6);
        assert_eq!(Queen::new(0, 7).sw_diagonal(bs), 0);

        assert_eq!(Queen::new(0, 0).sw_diagonal(bs), 7);
        assert_eq!(Queen::new(1, 1).sw_diagonal(bs), 7);
        assert_eq!(Queen::new(2, 2).sw_diagonal(bs), 7);
        assert_eq!(Queen::new(1, 0).sw_diagonal(bs), 8);
        assert_eq!(Queen::new(2, 0).sw_diagonal(bs), 9);
        assert_eq!(Queen::new(7, 0).sw_diagonal(bs), 14);
    }

    // #[test]
    // fn test_se_diagonal() {
    //     let bs = 8;
    //     assert_eq!(Queen::new(0, 1).se_diagonal(bs), 8);
    //     assert_eq!(Queen::new(0, 7).se_diagonal(bs), 14);

    //     assert_eq!(Queen::new(0, 0).se_diagonal(bs), 7);
    //     assert_eq!(Queen::new(1, 1).se_diagonal(bs), 7);
    //     assert_eq!(Queen::new(2, 2).se_diagonal(bs), 7);
    //     assert_eq!(Queen::new(1, 0).se_diagonal(bs), 6);
    //     assert_eq!(Queen::new(2, 0).se_diagonal(bs), 5);
    //     assert_eq!(Queen::new(7, 0).se_diagonal(bs), 0);
    //     assert_eq!(Queen::new(6, 1).se_diagonal(bs), 0);
    // }
}

// [][][][][][][][]
// [][][][][][][][]
// [][][][][][][][]
// [][][][][][][][]
// [][][][][][][][]
// [][][][][][][][]
// [][][][][][][][]
// [][][][][][][][]

// 01234567
// 12345678
// 23456789

