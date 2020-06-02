use crate::game::tetris::tetromino::Block;
use crate::settings::Renderable;
use crate::settings::Settings;
use std::cmp::min;
use std::collections::HashSet;
use std::iter::FromIterator;

pub struct TetrisField {
    field: Renderable,
}

impl TetrisField {
    pub fn new(settings: &Settings) -> TetrisField {
        assert!(settings.rows >= 4);
        assert!(settings.cols >= 4);
        let field = Renderable(vec![
            vec![0; settings.cols as usize];
            settings.rows as usize
        ]);
        TetrisField { field }
    }

    pub fn field(&self) -> Renderable {
        self.field.clone()
    }

    pub fn field_with_block(&self, block: &Block) -> Renderable {
        let mut field = self.field.clone();
        let shape = block.shape();
        for (y, row) in shape.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                let new_x = block.x + (x as i16);
                let new_y = block.y + (y as i16);
                if cell > 0 && self.in_bounds(new_x, new_y, false) {
                    field[new_y as usize][new_x as usize] = cell;
                }
            }
        }
        field
    }

    pub fn has_collision(&self, block: &Block) -> bool {
        let shape = block.shape();
        self.field
            .iter()
            .enumerate()
            .map(|(j, row)| {
                row.iter()
                    .enumerate()
                    .map(|(i, cell)| {
                        let inner_x = i as isize - block.x as isize;
                        let inner_y = j as isize - block.y as isize;
                        let x_in = 0 <= inner_x && inner_x < shape.len() as isize;
                        let y_in = 0 <= inner_y && inner_y < shape.len() as isize;
                        if x_in && y_in {
                            let element = shape[inner_y as usize][inner_x as usize];
                            if element > 0 && *cell > 0 {
                                Err(())
                            } else {
                                Ok(())
                            }
                        } else {
                            Ok(())
                        }
                    })
                    .collect()
            })
            .collect::<Result<(), ()>>()
            .is_err()
    }

    pub fn has_overflow(&self, block: &Block) -> bool {
        let shape = block.shape();
        for (j, row) in shape.iter().enumerate() {
            for (i, &cell) in row.iter().enumerate() {
                if cell > 0 && !self.in_bounds(block.x + (i as i16), block.y + (j as i16), true) {
                    return true;
                }
            }
        }
        false
    }

    pub fn in_bounds(&self, x: i16, y: i16, relaxed: bool) -> bool {
        (relaxed || 0 <= y)
            && y < self.field.len() as i16
            && 0 <= x
            && x < self.field[0].len() as i16
    }

    pub fn try_move(&self, block: &mut Block, x: i16, y: i16) -> bool {
        block.begin(block.x + x, block.y + y, block.rotation);
        let ok = !self.has_overflow(&block) && !self.has_collision(&block);
        block.end(ok);
        ok
    }

    pub fn try_rotate(&self, block: &mut Block) -> bool {
        for (x, y) in block.tetromino.wallkick(block.rotation).into_iter() {
            block.begin(
                block.x + x,
                block.y + y,
                block.rotation.next(block.tetromino.dir()),
            );
            let ok = !self.has_overflow(&block) && !self.has_collision(&block);
            block.end(ok);
            if ok {
                return ok;
            }
        }
        false
    }

    pub fn drop(&self, block: &mut Block) -> i16 {
        let altitude = self.altitude(block);
        block.y += altitude;
        altitude
    }

    pub fn altitude(&self, block: &Block) -> i16 {
        let shape = block.shape();
        let len = shape.len() as usize;
        let mut altitude = self.field.len() as i16;
        for i in 0..len {
            let mut lowest: Option<i16> = None;
            for (j, _) in shape.iter().enumerate() {
                if shape[j][i] > 0 {
                    lowest = Some(j as i16 + block.y)
                }
            }
            if let Some(lowest) = lowest {
                let mut highest = self.field.len() as i16;
                for y in 0..self.field.len() {
                    if self.field[y][(block.x + i as i16) as usize] > 0 {
                        highest = y as i16;
                        break;
                    }
                }
                altitude = min(altitude, highest - lowest - 1)
            }
        }
        altitude
    }

    // Returns number of dropped line
    pub fn consume(&mut self, block: Block) -> u16 {
        let shape = block.shape();
        let mut affected_lines = HashSet::new();
        for (j, row) in shape.iter().enumerate() {
            for (i, &cell) in row.iter().enumerate() {
                let x = block.x + (i as i16);
                let y = block.y + (j as i16);
                if cell > 0 && self.in_bounds(x, y, false) {
                    affected_lines.insert(y as u16);
                    self.field[y as usize][x as usize] = cell;
                }
            }
        }
        let mut affected_lines = Vec::from_iter(affected_lines);
        affected_lines.sort();
        self.check_filled(affected_lines)
    }

    // Returns number of dropped line
    pub fn check_filled(&mut self, lines: Vec<u16>) -> u16 {
        let mut drop = vec![];
        'lines: for line in lines.iter() {
            for i in 0..self.field[*line as usize].len() {
                if self.field[*line as usize][i] == 0 {
                    continue 'lines;
                }
            }
            drop.push(*line);
        }
        for line in drop.iter() {
            self.field.remove(*line as usize);
            let len = self.field[0].len();
            self.field.insert(0, vec![0; len])
        }
        drop.len() as u16
    }
}
