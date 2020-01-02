mod utils;

use std::collections::LinkedList;
use rand::Rng;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static mut ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Point2D {
    x: f64,
    y: f64
}

struct Vector2D {
    x: f64,
    y: f64
}

struct Particle {
    id: i32,
    radius: f64,
    location: Point2D,
    direction: Vector2D
}

static mut POINTS: LinkedList<Particle> = LinkedList::new();
static MAX_POINTS: i32 = 500;
static MAX_X: f64 = 600.0;
static MAX_Y: f64 = 600.0;

////////// ----------------------------------------------------------------------
////////// PUBLIC & EXTERNAL FUNCTIONS    ---------------------------------------
////////// ----------------------------------------------------------------------

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
        let location = Point2D { x: rng.gen::<f64>() * MAX_X, y: rng.gen::<f64>() * MAX_Y };
        let direction = Vector2D { x: rng.gen::<f64>() * 2.0, y: rng.gen::<f64>() % 2.0 };
        log(&mut format!("x: {x}, y: {y}", x=location.x, y=location.y));
        unsafe {
            POINTS.push_back(Particle { id, location, direction, radius: rng.gen::<f64>() * 5.0 + 2.0 })
        }
    }
    log("Setup done.");
}

#[wasm_bindgen]
pub fn render() {
    move_points();
    draw_points();
}

////////// ----------------------------------------------------------------------
////////// VECTOR OPERATION       -----------------------------------------------
////////// ----------------------------------------------------------------------

fn invert(value: f64) -> f64 {
    return value * -1.0;
}

fn move_point(particle: &mut Particle) {
    particle.location.x += particle.direction.x;
    particle.location.y += particle.direction.y;
}

fn subtract(v1: &Vector2D, v2: &Vector2D) -> Vector2D {
    let x = v1.x - v2.x;
    let y = v1.y - v2.y;
    return Vector2D { x, y };
}

fn add(v1: &Vector2D, v2: &Vector2D) -> Vector2D {
    let x = v1.x + v2.x;
    let y = v1.y + v2.y;
    return Vector2D { x, y };
}

fn divide(v1: &Vector2D, value: f64) -> Vector2D {
    return Vector2D { x: v1.x / value, y: v1.y / value };
}

fn length(v: &Vector2D) -> f64 {
    return ((v.x*v.x) + (v.y*v.x)).sqrt()
}

fn dot(v1: &Vector2D, v2: &Vector2D) -> f64 {
    let x = v1.x * v2.x;
    let y = v1.y * v2.y;
    return x + y;
}

fn normalize(v: &Vector2D) -> Vector2D {
    let len2 = (v.x * v.x) + (v.y * v.y);
    if len2 > 0.0 {
        let invLen = 1.0 / len2.sqrt(); 
        return Vector2D { x: v.x * invLen, y: v.y * invLen };
    }

    return Vector2D { x: v.x, y: v.y };
}

////////// ----------------------------------------------------------------------
////////// COLLISION DETECTION    -----------------------------------------------
////////// ----------------------------------------------------------------------

fn check_colide_edge(particle: &mut Particle) {
    let next_x = particle.location.x + particle.direction.x;
    let next_y = particle.location.y + particle.direction.y;

    if next_x > MAX_X || next_x < 0.0 {
        particle.direction.x = invert(particle.direction.x);
    }

    if next_y > MAX_Y || next_y < 0.0 {
        particle.direction.y = invert(particle.direction.y);            
    }
}

fn check_particle_colision(p1: &Particle, p2: &Particle) -> bool {
    let x1 = p1.location.x;
    let y1 = p1.location.y;
    let x2 = p2.location.x;
    let y2 = p2.location.y;
    return (( x2-x1 ) * ( x2-x1 )  + ( y2-y1 ) * ( y2-y1 )).sqrt() < p1.radius + p2.radius;
}

fn distance(p1: &mut Point2D, p2: &mut Point2D) -> f64 {
    let x = p1.x - p2.x;
    let y = p1.y - p2.y;
    return ((x*x) + (y*y)).sqrt();
}

fn colide_particles(p1: &mut Particle, p2: &mut Particle) {
    // adjust p1 location
    let dist = distance(p1.location, p2.location);
    let diff = dist - p1.radius - p2.radius;
    p1.location.x = p1.location.x + diff;
    p1.location.y = p1.location.y + diff;

    // change direction
    let dotv = dot(&p1.direction, &p2.direction);
    let v1 = Vector2D { x: p1.direction.x * dotv, y: p1.direction.y * dotv };
    let v2 = Vector2D { x: p2.direction.x * dotv, y: p2.direction.y * dotv };
    p1.direction = normalize(&v1);
    p2.direction = normalize(&v2);

    // p1.direction.x = invert(p1.direction.x);
    // p1.direction.y = invert(p1.direction.y);
    // p2.direction.x = invert(p2.direction.x);
    // p2.direction.y = invert(p2.direction.y);
    move_point(p1);
}

fn check_particle_colisions(particle: &mut Particle) {
    unsafe {
        for particle_to_compare in POINTS.iter_mut() {
            if particle.id < particle_to_compare.id && check_particle_colision(particle, particle_to_compare) {
                colide_particles(particle, particle_to_compare);
            }
        }
    }
}

////////// ----------------------------------------------------------------------
////////// MOTION           -----------------------------------------------------
////////// ----------------------------------------------------------------------

fn move_points() {
    unsafe {
        for mut particle in POINTS.iter_mut() {
            check_colide_edge(particle);    
            move_point(&mut particle);
            check_particle_colisions(particle);
        }
    }
}


fn draw_point(particle: &mut Particle, context: & web_sys::CanvasRenderingContext2d) {
    context.begin_path();
    context.arc(particle.location.x, particle.location.y, particle.radius, 0.0, 360.0).ok();
    if particle.id % 3 == 0 {
        context.set_fill_style(&JsValue::from_str("rgb(255, 0, 0)"));
    } else if particle.id % 3 == 1 {
        context.set_fill_style(&JsValue::from_str("rgb(0, 255, 0)"));
    } else if particle.id % 3 == 2 {
        context.set_fill_style(&JsValue::from_str("rgb(0, 0, 255)"));
    }
    context.fill();

    context.begin_path();
    context.set_stroke_style(&JsValue::from_str("rgb(255, 255, 255)"));
    context.move_to(particle.location.x, particle.location.y);
    context.line_to(particle.location.x+(particle.direction.x*4.0), particle.location.y+(particle.direction.y*4.0));
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
        context.set_fill_style(&JsValue::from_str("rgb(0, 0, 0)"));
        context.fill_rect(0.0, 0.0, MAX_X, MAX_Y);
        context.stroke();
        for particle in POINTS.iter_mut() {
            draw_point(particle, &context)
        }
    }
}
