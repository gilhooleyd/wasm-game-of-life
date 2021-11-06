mod utils;

use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

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

    pub fn toggle_cell(&mut self, width: u32, height: u32) {
        let idx = self.get_index(width, height);
        if self.cells[idx] == Cell::Alive {
            self.cells[idx] = Cell::Dead;
        } else {
            self.cells[idx] = Cell::Alive;
        }
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
}

pub struct Canvas {
    canvas: web_sys::HtmlCanvasElement,
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
            canvas: canvas,
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
    utils::set_panic_hook();
    log!("Starting our Game of Life!");

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let canvas = Rc::new(RefCell::new(Canvas::new(Universe::new(), canvas)));
    canvas.borrow_mut().draw_grid();
    canvas.borrow_mut().draw_cells();

    let is_running = Rc::new(RefCell::new(false));

    // Create the animation callback.
    let animation_callback = Rc::new(RefCell::new(None));
    {
        let is_running = is_running.clone();
        let canvas = canvas.clone();

        let callback = animation_callback.clone();
        *animation_callback.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            canvas.borrow_mut().universe.tick();
            canvas.borrow_mut().draw_grid();
            canvas.borrow_mut().draw_cells();

            // Schedule ourself for another requestAnimationFrame callback.
            if *is_running.borrow() {
                request_animation_frame(callback.borrow().as_ref().unwrap());
            }
        }) as Box<dyn FnMut()>));
        request_animation_frame(animation_callback.borrow().as_ref().unwrap());
    }

    // Create the play button callback.
    {
        let animation_callback = animation_callback.clone();
        let play_callback = Closure::wrap(Box::new(move || {
            let is_running = is_running.clone();
            let is_running_val = *is_running.borrow();
            *is_running.borrow_mut() = !is_running_val;
            if !is_running_val {
                request_animation_frame(animation_callback.borrow().as_ref().unwrap());
            }
        }) as Box<dyn FnMut()>);
        document
            .get_element_by_id("play-pause")
            .expect("should have #play-pause on the page")
            .dyn_ref::<web_sys::HtmlElement>()
            .expect("#play-pause be an `HtmlElement`")
            .set_onclick(Some(play_callback.as_ref().unchecked_ref()));
        play_callback.forget();
    }

    // Create the click callback.
    {
        let my_canvas = canvas.clone();
        let click_callback = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let row = event.offset_y() as u32 / (my_canvas.borrow().cell_size + 1);
            let col = event.offset_x() as u32 / (my_canvas.borrow().cell_size + 1);
            my_canvas.borrow_mut().universe.toggle_cell(row, col);
            my_canvas.borrow_mut().draw_cells();
        }) as Box<dyn FnMut(_)>);
        canvas
            .borrow_mut()
            .canvas
            .add_event_listener_with_callback("click", click_callback.as_ref().unchecked_ref())
            .unwrap();
        click_callback.forget();
    }
}
