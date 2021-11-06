mod utils;

use std::cell::RefCell;
use std::f64;
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
    pub fn cell(&self, width: u32, height: u32) -> Cell {
        let idx = self.get_index(width, height);
        self.cells[idx]
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    pub fn new() -> Universe {
        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

pub struct Canvas {
    context: web_sys::CanvasRenderingContext2d,
    universe: Universe,

    cell_size: u32,
    grid_color: wasm_bindgen::JsValue,
    dead_color: wasm_bindgen::JsValue,
    alive_color: wasm_bindgen::JsValue,
}
impl Canvas {
    fn new(universe: Universe, canvas: web_sys::HtmlCanvasElement) -> Canvas {
        let cell_size = 10;
        canvas.set_height((cell_size + 1) * universe.height() + 1);
        canvas.set_width((cell_size + 1) * universe.width() + 1);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        Canvas {
            context: context,
            cell_size: cell_size,
            grid_color: wasm_bindgen::JsValue::from_str("#CCCCCC"),
            dead_color: wasm_bindgen::JsValue::from_str("#FFFFFF"),
            alive_color: wasm_bindgen::JsValue::from_str("#000000"),
            universe: universe,
        }
    }
    fn draw_grid(&self) {
        let cell_size = &self.cell_size;
        let universe = &self.universe;
        let context = &self.context;

        self.context.begin_path();
        self.context.set_stroke_style(&self.grid_color);

        for i in 0..self.universe.width() {
            context.move_to((i * (cell_size + 1)) as f64, 0 as f64);
            context.line_to(
                (i * (cell_size + 1) + 1) as f64,
                ((cell_size + 1) * universe.height + 1) as f64,
            );
        }

        for i in 0..universe.height() {
            context.move_to(0 as f64, (i * (cell_size + 1)) as f64);
            context.line_to(
                (i * (cell_size + 1) * universe.width() + 1) as f64,
                (i * (cell_size + 1) + 1) as f64,
            );
        }

        context.stroke();
    }

    fn draw_cells(&self) {
        self.context.begin_path();

        let cell_size = &self.cell_size;
        for row in 0..self.universe.height() {
            for col in 0..self.universe.width() {
                if self.universe.cell(row, col) == Cell::Dead {
                    self.context.set_fill_style(&self.dead_color);
                } else {
                    self.context.set_fill_style(&self.alive_color);
                }

                self.context.fill_rect(
                    (col * (cell_size + 1) + 1) as f64,
                    (row * (cell_size + 1) + 1) as f64,
                    *cell_size as f64,
                    *cell_size as f64,
                );
            }
        }
        self.context.stroke();
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen(start)]
pub fn start() {

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let mut canvas = Canvas::new(Universe::new(), canvas);
    canvas.draw_grid();
    canvas.draw_cells();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        canvas.universe.tick();
        canvas.draw_grid();
        canvas.draw_cells();

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}
