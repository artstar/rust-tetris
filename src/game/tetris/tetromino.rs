use crate::bootstrap::{Renderable, Settings};

pub struct Block {
    pub tetromino: Tetromino,
    pub rotation: Rotation,
    pub x: i16,
    pub y: i16,
    saved_rotation: Option<Rotation>,
    saved_x: Option<i16>,
    saved_y: Option<i16>,
}

impl Block {
    pub fn spawn(tetromino: Tetromino, settings: &Settings) -> Block {
        let rotation = Rotation::Default;
        let y = -1;
        let x = settings.cols as i16 / 2 - (tetromino.shape().len() as i16 + 1) / 2;
        Block {
            tetromino,
            rotation,
            x,
            y,
            saved_rotation: None,
            saved_x: None,
            saved_y: None,
        }
    }

    pub fn shape(&self) -> Renderable {
        let shape = self.tetromino.shape();
        let len = shape.len() as u8;
        let mut result = Renderable(vec![vec![0u8; len as usize]; len as usize]);
        let transform: Box<dyn Fn(u8, u8) -> (u8, u8)> = match self.rotation {
            Rotation::Default => Box::new(|x, y| (x, y)),
            Rotation::CCW => Box::new(|x, y| (y, len - x - 1)),
            Rotation::Reverse => Box::new(|x, y| (len - x - 1, len - y - 1)),
            Rotation::CW => Box::new(|x, y| (len - y - 1, x)),
        };
        for i in 0..len {
            for j in 0..len {
                let (x, y) = &transform(i, j);
                result[i as usize][j as usize] = shape[*x as usize][*y as usize]
            }
        }
        result
    }

    pub fn begin(&mut self, x: i16, y: i16, rotation: Rotation) {
        self.saved_x = Some(self.x);
        self.saved_y = Some(self.y);
        self.saved_rotation = Some(self.rotation);
        self.x = x;
        self.y = y;
        self.rotation = rotation;
    }

    pub fn commit(&mut self) {
        self.saved_x = None;
        self.saved_y = None;
        self.saved_rotation = None;
    }

    pub fn revert(&mut self) {
        if let Some(x) = self.saved_x {
            self.x = x;
            self.saved_x = None;
        }
        if let Some(y) = self.saved_y {
            self.y = y;
            self.saved_y = None;
        }
        if let Some(rotation) = self.saved_rotation {
            self.rotation = rotation;
            self.saved_rotation = None;
        }
    }

    pub fn end(&mut self, commit: bool) {
        if commit {
            self.commit()
        } else {
            self.revert()
        };
    }
}

#[derive(Copy, Clone)]
pub enum Direction {
    None,
    Half,
    Full,
}

#[derive(Copy, Clone)]
pub enum Rotation {
    Default,
    CW,
    Reverse,
    CCW,
}

impl Rotation {
    pub fn next(self, dir: Direction) -> Self {
        match (dir, self) {
            (Direction::None, Rotation::Default) => Rotation::Default,

            (Direction::Half, Rotation::Default) => Rotation::CCW,
            (Direction::Half, Rotation::CCW) => Rotation::Default,

            (Direction::Full, Rotation::Default) => Rotation::CW,
            (Direction::Full, Rotation::CW) => Rotation::Reverse,
            (Direction::Full, Rotation::Reverse) => Rotation::CCW,
            (Direction::Full, Rotation::CCW) => Rotation::Default,
            _ => unreachable!(),
        }
    }
}

pub enum Tetromino {
    I(I),
    T(T),
    J(J),
    L(L),
    S(S),
    Z(Z),
    O(O),
}

impl Tetromino {
    pub fn shape(&self) -> &[&[u8]] {
        match &self {
            Self::I(_) => I::SHAPE,
            Self::T(_) => T::SHAPE,
            Self::L(_) => L::SHAPE,
            Self::J(_) => J::SHAPE,
            Self::S(_) => S::SHAPE,
            Self::Z(_) => Z::SHAPE,
            Self::O(_) => O::SHAPE,
        }
    }

    pub fn dir(&self) -> Direction {
        match &self {
            Self::T(_) | Self::L(_) | Self::J(_) => Direction::Full,
            Self::I(_) | Self::S(_) | Self::Z(_) => Direction::Half,
            Self::O(_) => Direction::None,
        }
    }

    pub fn wallkick(&self, rotation: Rotation) -> Vec<(i16, i16)> {
        match &self {
            Self::T(_) | Self::L(_) | Self::J(_) => match rotation {
                Rotation::Default => vec![(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                Rotation::CW => vec![(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                Rotation::Reverse => vec![(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                Rotation::CCW => vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            },
            Self::S(_) | Self::Z(_) => match rotation {
                Rotation::Default => vec![(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                Rotation::CCW => vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                _ => unreachable!(),
            },
            Self::I(_) => match rotation {
                Rotation::Default => vec![(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                Rotation::CCW => vec![(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                _ => unreachable!(),
            },
            Self::O(_) => vec![(0, 0)],
        }
    }

    pub fn preview(&self) -> Renderable {
        let mut v: Renderable = Renderable(
            self.shape()
                .iter()
                .map(|v| {
                    let mut vec = v.to_vec();
                    if vec.len() < 3 {
                        vec.insert(0, 0)
                    }
                    vec.resize(4, 0);
                    vec
                })
                .collect(),
        );
        if v.len() < 4 {
            v.insert(0, vec![0u8, 0u8, 0u8, 0u8]);
        }
        v.resize(4, vec![0u8, 0u8, 0u8, 0u8]);
        v
    }
}

pub trait Figure {
    const SHAPE: &'static [&'static [u8]];
}

pub struct I();

impl Figure for I {
    #[rustfmt::skip]
    const SHAPE: &'static [&'static [u8]] = &[
        &[0, 0, 0, 0],
        &[1, 1, 1, 1],
        &[0, 0, 0, 0],
        &[0, 0, 0, 0]
    ];
}

impl From<I> for Tetromino {
    fn from(figure: I) -> Self {
        Tetromino::I(figure)
    }
}

pub struct T();

impl Figure for T {
    #[rustfmt::skip]
    const SHAPE: &'static [&'static [u8]] = &[
        &[0, 2, 0],
        &[2, 2, 2],
        &[0, 0, 0]
    ];
}

impl From<T> for Tetromino {
    fn from(figure: T) -> Self {
        Tetromino::T(figure)
    }
}

pub struct J();

impl Figure for J {
    #[rustfmt::skip]
    const SHAPE: &'static [&'static [u8]] = &[
        &[3, 0, 0],
        &[3, 3, 3],
        &[0, 0, 0]
    ];
}

impl From<J> for Tetromino {
    fn from(figure: J) -> Self {
        Tetromino::J(figure)
    }
}

pub struct L();

impl Figure for L {
    #[rustfmt::skip]
    const SHAPE: &'static [&'static [u8]] = &[
        &[0, 0, 4],
        &[4, 4, 4],
        &[0, 0, 0]
    ];
}

impl From<L> for Tetromino {
    fn from(figure: L) -> Self {
        Tetromino::L(figure)
    }
}

pub struct S();

impl Figure for S {
    #[rustfmt::skip]
    const SHAPE: &'static [&'static [u8]] = &[
        &[0, 5, 5],
        &[5, 5, 0],
        &[0, 0, 0]
    ];
}

impl From<S> for Tetromino {
    fn from(figure: S) -> Self {
        Tetromino::S(figure)
    }
}

pub struct Z();

impl Figure for Z {
    #[rustfmt::skip]
    const SHAPE: &'static [&'static [u8]] = &[
        &[6, 6, 0],
        &[0, 6, 6],
        &[0, 0, 0]
    ];
}

impl From<Z> for Tetromino {
    fn from(figure: Z) -> Self {
        Tetromino::Z(figure)
    }
}

pub struct O();

impl Figure for O {
    #[rustfmt::skip]
    const SHAPE: &'static [&'static [u8]] = &[
        &[7, 7],
        &[7, 7]
    ];
}

impl From<O> for Tetromino {
    fn from(figure: O) -> Self {
        Tetromino::O(figure)
    }
}
