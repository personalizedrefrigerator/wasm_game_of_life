use wasm_bindgen::prelude::*;

const DEFAULT_SQUARE_SIZE: f64 = 8.0;
const DEFAULT_SPACING: f64 = 1.0;

#[wasm_bindgen(start)]
pub fn main() {
    // Better error logging:
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    buffered_cells_: Vec<Cell>,

    cells: Vec<Cell>,
    width: u32,
    height: u32,

    square_size_px: f64,
    square_spacing_px: f64,
}

#[wasm_bindgen]
impl Universe {
    /// Apply the rules of the game of life once to all cells in this.
    pub fn tick(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                let idx = self.get_cell_idx(x, y);

                self.buffered_cells_[idx] = match (self.cells[idx], self.get_live_neighbor_count(x, y)) {
                    (_, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (_, 3) => Cell::Alive,
                    (_, x) if x > 3 => Cell::Dead,
                    (otherwise, _) => otherwise,
                }
            }
        }

        std::mem::swap(&mut self.buffered_cells_, &mut self.cells);
    }

    pub fn get_cell_at(&self, x: u32, y: u32) -> Cell {
        self.cells[self.get_cell_idx(x, y)]
    }

    /// Sets the cell at ([x], [y]) to [cell_type], where
    /// x ∈ [0, self.width) and y ∈ [0, self.height).
    pub fn set_cell_at(&mut self, x: u32, y: u32, cell_type: Cell) {
        if x < self.width && y < self.height {
            let idx = self.get_cell_idx(x, y);
            self.cells[idx] = cell_type;
        }
    }

    pub fn toggle_cell_at(&mut self, x: u32, y: u32) {
        let new_value = match self.get_cell_at(x, y) {
            Cell::Alive => Cell::Dead,
            Cell::Dead => Cell::Alive,
        };

        self.set_cell_at(x, y, new_value);
    }

    /// Toggles all cells along the line between (x1, y1) and (x2, y2), but cells
    /// at (x1, y1) and (x2, y2) remain the same.
    pub fn toggle_cells_between(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        if x1 == x2 && y1 == y2 {
            return;
        }

        let mut delta_y = (y2 as f64) - (y1 as f64);
        let mut delta_x = (x2 as f64) - (x1 as f64);

        let x_is_param = delta_y.abs() < delta_x.abs();
        let y_is_param = !x_is_param;

        let (x1, x2, y1, y2) = if (x2 < x1 && x_is_param) || (y2 < y1 && y_is_param) {
            delta_x = -delta_x;
            delta_y = -delta_y;

            (x2, x1, y2, y1)
        } else {
            (x1, x2, y1, y2)
        };

        if x_is_param {
            let mut y = y1 as f64;

            for x in (x1 + 1)..x2 {
                y += delta_y / delta_x;
                self.toggle_cell_at(x, y.round() as u32);
            }
        } else {
            let mut x = x1 as f64;

            for y in (y1 + 1)..y2 {
                x += delta_x / delta_y;
                self.toggle_cell_at(x.round() as u32, y);
            }
        }
    }

    /// Sets all cells to Cell::Dead
    pub fn clear(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.set_cell_at(x, y, Cell::Dead);
            }
        }
    }

    pub fn fill_cells(&self, cell_type: Cell, ctx: &web_sys::CanvasRenderingContext2d) {
        for x in 0..self.width {
            for y in 0..self.height {
                let cell = self.get_cell_at(x, y);

                if cell == cell_type {
                    continue;
                }

                let square_x = (x as f64) * (self.square_size_px + self.square_spacing_px) + self.square_spacing_px;
                let square_y = (y as f64) * (self.square_size_px + self.square_spacing_px) + self.square_spacing_px;

                ctx.fill_rect(square_x, square_y, self.square_size_px, self.square_size_px);
            }
        }
    }

    pub fn set_square_size(&mut self, size: f64) {
        self.square_size_px = size;
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }

    pub fn get_square_size(&self) -> f64 { self.square_size_px }
    pub fn get_square_spacing(&self) -> f64 { self.square_spacing_px }

    /// Create a new universe with initial data based on that in [template].
    pub fn resize_to(&mut self, width: u32, height: u32) {
        let mut cells: Vec<Cell> = (0..width*height)
                .map(|i: u32| { ( i % width, i / width ) })
                .map(|(x, y)| {
                    self.get_cell_at(x, y)
                })
                .collect();
        let mut background_cells = cells.clone();

        self.width = width;
        self.height = height;
        std::mem::swap(&mut self.cells, &mut cells);
        std::mem::swap(&mut self.buffered_cells_, &mut background_cells);
    }

    pub fn new(width: u32, height: u32) -> Universe {
        let cells: Vec<Cell> = (0..width * height)
                .map(|i| {
                    if i % 2 == 0 || i % 7 == 0 {
                        Cell::Alive
                    } else {
                        Cell::Dead
                    }
                })
                .collect();
        let background_cells = cells.clone();

        Universe {
            buffered_cells_: background_cells,
            cells,
            width,
            height,

            square_size_px: DEFAULT_SQUARE_SIZE,
            square_spacing_px: DEFAULT_SPACING,
        }
    }
}

// Private impl
impl Universe {
    fn get_cell_idx(&self, x: u32, y: u32) -> usize {
        let x = x % self.width;
        let y = y % self.height;

        (y * self.width + x) as usize
    }

    fn get_live_neighbor_count(&self, x: u32, y: u32) -> u32 {
        let mut count = 0;

        // Note that everything is modulo self.width or self.height.
        // As such, x + self.width - 1 \equiv x - 1 (mod self.width),
        //    but x + self.width - 1 avoids unsigned integer wrapping.
        for dx in [self.width - 1, 0, 1].iter().cloned() {
            for dy in [self.height - 1, 0, 1].iter().cloned() {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let x = (x + dx) % self.width;
                let y = (y + dy) % self.height;

                count += match self.get_cell_at(x, y) {
                    Cell::Dead => 0,
                    Cell::Alive => 1,
                };
            }
        }

        count
    }
}

