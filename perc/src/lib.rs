#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

use rand::Rng;
use std::clone::Clone;

/// Represents a grid of boolean values.

#[derive(Clone)]
pub struct BoolGrid {
    width: usize,
    height: usize,
    vacancy: f64,
    field: Vec<Vec<bool>>,
}

impl BoolGrid {
    /// Creates a new grid with all values initialized as `false`.
    ///
    /// # Arguments
    ///
    /// * `width` - grid width.
    /// * `height` - grid height.
    pub fn new(width: usize, height: usize) -> Self {
        let field = vec![vec![false; height]; width];
        let vacancy = 1.0_f64;
        Self {
            width,
            height,
            vacancy,
            field,
        }
    }

    /// Creates a new grid with every value initialized randomly.
    ///
    /// # Arguments
    ///
    /// * `width` - grid width.
    /// * `height` - grid height.
    /// * `vacancy` - probability of any given value being equal
    /// to `false`.
    pub fn random(width: usize, height: usize, vacancy: f64) -> Self {
        let mut field = vec![vec![true; height]; width];
        let mut rng = rand::thread_rng();
        for x in 0..width {
            for y in 0..height {
                field[x][y] = rng.gen_range(0.0..1.0) > vacancy;
            }
        }
        Self {
            width,
            height,
            vacancy,
            field,
        }
    }

    /// Returns grid width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns grid height.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the current value of a given cell.
    /// The caller must ensure that `x` and `y` are valid.
    ///
    /// # Arguments
    ///
    /// * `x` - must be >= 0 and < grid width.
    /// * `y` - must be >= 0 and < grid height.
    ///
    /// # Panics
    ///
    /// If `x` or `y` is out of bounds, this method may panic
    /// (or return incorrect result).
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.check_coordinates(x, y);
        self.field[x][y]
    }

    /// Sets a new value to a given cell.
    /// The caller must ensure that `x` and `y` are valid.
    ///
    /// # Arguments
    ///
    /// * `x` - must be >= 0 and < grid width.
    /// * `y` - must be >= 0 and < grid height.
    ///
    /// # Panics
    ///
    /// If `x` or `y` is out of bounds, this method may panic
    /// (or set value to some other unspecified cell).
    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.check_coordinates(x, y);
        self.field[x][y] = value;
    }

    fn check_coordinates(&self, x: usize, y: usize) {
        if x >= self.width {
            panic!("Incorrect value of the x coordinate.")
        }

        if y >= self.height {
            panic!("Incorrect value of the y coordinate.")
        }
    }

    pub fn print(&self) {
        for x in 0..self.width {
            for y in 0..self.height {
                print!("{}", if self.field[x][y] { "#" } else { "." });
            }
            println!();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Returns `true` if the given grid percolates. That is, if there is a path
/// from any cell with `y` == 0 to any cell with `y` == `height` - 1.
/// If the grid is empty (`width` == 0 or `height` == 0), it percolates.
pub fn percolates(grid: &BoolGrid) -> bool {
    if grid.width() == 0 || grid.height() == 0 {
        return true;
    }
    let mut occupied = grid.clone();
    for x in 0..grid.width() {
        if !occupied.get(x, 0) && find_path(&mut occupied, x, 0) {
            return true;
        }
    }
    return false;
}

fn find_path(grid: &mut BoolGrid, x: usize, y: usize) -> bool {
    grid.set(x, y, true);
    if y == grid.height() - 1
        || check_free(grid, x, y + 1)
        || x != 0 && check_free(grid, x - 1, y)
        || x < grid.width() - 1 && check_free(grid, x + 1, y)
        || y != 0 && check_free(grid, x, y - 1)
    {
        grid.set(x, y, false);
        return true;
    }
    false
}
////////////////////////////////////////////////////////////////////////////////

fn check_free(grid: &mut BoolGrid, x: usize, y: usize) -> bool {
    if grid.get(x, y) {
        return false;
    }
    find_path(grid, x, y)
}

const N_TRIALS: u64 = 10000;

/// Returns an estimate of the probability that a random grid with given
/// `width, `height` and `vacancy` probability percolates.
/// To compute an estimate, it runs `N_TRIALS` of random experiments,
/// in each creating a random grid and checking if it percolates.
pub fn evaluate_probability(width: usize, height: usize, vacancy: f64) -> f64 {
    let mut perc_count = 0;
    for _ in 0..N_TRIALS {
        let grid = BoolGrid::random(width, height, vacancy);
        if percolates(&grid) {
            perc_count += 1;
        }
    }
    return perc_count as f64 / N_TRIALS as f64;
}
