mod utils;

use std::collections::LinkedList;
use rand::Rng;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static mut ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Point2D {
    x: i32,
    y: i32
}

struct Vector2D {
    location: Point2D,
    direction: Point2D
}

struct Particle {
    id: i32,
    radius: i32,
    vector: Vector2D
}

static mut POINTS: LinkedList<Particle> = LinkedList::new();
static MAX_POINTS: i32 = 200;
static MAX_X: i32 = 300;
static MAX_Y: i32 = 300;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn setup() {
    log("Initializing objects.");
    let mut rng = rand::thread_rng();
    for id in 0..MAX_POINTS {
        let location = Point2D { x: rng.gen::<i32>() % MAX_X, y: rng.gen::<i32>() % MAX_Y };
        let direction = Point2D { x: rng.gen::<i32>() % 2, y: rng.gen::<i32>() % 2 };
        unsafe {
            let vector = Vector2D { location, direction };
            POINTS.push_back(Particle { id, vector, radius: 2 })
        }
    }
    log("Setup done.");
}

fn check_colide_edge(particle: &mut Particle) {
    let mut vector = &mut particle.vector;
    let next_x = vector.location.x + vector.direction.x;
    let next_y = vector.location.y + vector.direction.y;

    if next_x > MAX_X || next_x < 0 {
        vector.direction.x = invert(vector.direction.x);
    }

    if next_y > MAX_Y || next_y < 0 {
        vector.direction.y = invert(vector.direction.y);            
    }
}

fn invert(value: i32) -> i32 {
     return value * -1;
}

fn move_point(vector: &mut Vector2D) {
    vector.location.x += vector.direction.x;
    vector.location.y += vector.direction.y;
}

fn check_point_colision(particle: &mut Particle) {
    let mut vector = &mut particle.vector;
    unsafe {
        for particle_to_compare in POINTS.iter_mut() {
            if particle.id < particle_to_compare.id {
                let mut vector_to_compare = &mut particle_to_compare.vector;
                let x1 = vector.location.x;
                let y1 = vector.location.y;
                let x2 = vector_to_compare.location.x;
                let y2 = vector_to_compare.location.y;
                if f64::from( ( x2-x1 ) * ( x2-x1 )  + ( y2-y1 ) * ( y2-y1 ) ).sqrt() < f64::from(particle.radius + particle_to_compare.radius) {
                    vector.direction.x = invert(vector.direction.x);
                    vector.direction.y = invert(vector.direction.y);
                    vector_to_compare.direction.x = invert(vector_to_compare.direction.x);
                    vector_to_compare.direction.y = invert(vector_to_compare.direction.y);
                    move_point(&mut vector);
                }
            }
        }
    }
}

fn move_points() {
    unsafe {
        for particle in POINTS.iter_mut() {
            check_colide_edge(particle);    
            move_point(&mut particle.vector);
            check_point_colision(particle);
        }
    }
}

fn draw_point(particle: &mut Particle, context: & web_sys::CanvasRenderingContext2d) {
    let vector = &particle.vector;
    context.begin_path();
    context.arc(f64::from(vector.location.x), f64::from(vector.location.y), f64::from(particle.radius), 0.0, 360.0).ok();
    if particle.id % 3 == 0 {
        context.set_fill_style(&JsValue::from_str("rgb(255, 0, 0)"));
    } else if particle.id % 3 == 1 {
        context.set_fill_style(&JsValue::from_str("rgb(0, 255, 0)"));
    } else if particle.id % 3 == 2 {
        context.set_fill_style(&JsValue::from_str("rgb(0, 0, 255)"));
    }
    context.fill();

    context.begin_path();
    context.set_stroke_style(&JsValue::from_str("rgb(0, 0, 0)"));
    context.move_to(f64::from(vector.location.x), f64::from(vector.location.y));
    context.line_to(f64::from(vector.location.x+(vector.direction.x*4)), f64::from(vector.location.y+(vector.direction.y*4)));
    context.stroke();
}

fn draw_points() {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .get_element_by_id("dot-canvas")
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    unsafe {
        context.begin_path();
        context.clear_rect(0.0, 0.0, f64::from(MAX_X), f64::from(MAX_Y));
        context.stroke();
        for particle in POINTS.iter_mut() {
            draw_point(particle, &context)
        }
    }
}

#[wasm_bindgen]
pub fn render() {
    move_points();
    draw_points();
}
