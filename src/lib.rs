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
    location: Point2D,
    direction: Point2D
}

struct Particle {
    id: i32,
    radius: f64,
    vector: Vector2D
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
        let direction = Point2D { x: rng.gen::<f64>() * 2.0, y: rng.gen::<f64>() % 2.0 };
        log(&mut format!("x: {x}, y: {y}", x=location.x, y=location.y));
        unsafe {
            let vector = Vector2D { location, direction };
            POINTS.push_back(Particle { id, vector, radius: rng.gen::<f64>() * 4.0 + 2.0 })
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

fn move_point(vector: &mut Vector2D) {
    vector.location.x += vector.direction.x;
    vector.location.y += vector.direction.y;
}

fn subtract(v1: &Point2D, v2: &Point2D) -> Point2D {
    let x = v1.x - v2.x;
    let y = v1.y - v2.y;
    return Point2D { x, y };
}

fn divide(v1: &Point2D, value: f64) -> Point2D {
    return Point2D { x: v1.x / value, y: v1.y / value };
}

fn distance(v1: &Point2D, v2: &Point2D) -> f64 {
    let x = v1.x + v2.x;
    let y = v1.y + v2.y;
    return ((x*x) - (y*y)).sqrt()
}

////////// ----------------------------------------------------------------------
////////// COLLISION DETECTION    -----------------------------------------------
////////// ----------------------------------------------------------------------

fn check_colide_edge(particle: &mut Particle) {
    let mut vector = &mut particle.vector;
    let next_x = vector.location.x + vector.direction.x;
    let next_y = vector.location.y + vector.direction.y;

    if next_x > MAX_X || next_x < 0.0 {
        vector.direction.x = invert(vector.direction.x);
    }

    if next_y > MAX_Y || next_y < 0.0 {
        vector.direction.y = invert(vector.direction.y);            
    }
}

fn check_particle_colision(p1: &Particle, p2: &Particle) -> bool {
    let x1 = p1.vector.location.x;
    let y1 = p1.vector.location.y;
    let x2 = p2.vector.location.x;
    let y2 = p2.vector.location.y;
    return (( x2-x1 ) * ( x2-x1 )  + ( y2-y1 ) * ( y2-y1 )).sqrt() < p1.radius + p2.radius;
}

fn colide_particles(p1: &mut Particle, p2: &mut Particle) {
    let p0 = Point2D { x: 0.0, y: 0.0 };
    let velocity_p1 = distance(&p0, &p1.vector.direction);
    let velocity_p2 = distance(&p0, &p2.vector.direction);
    let optimizedP: f64 = velocity_p1 - velocity_p2;

    // log(&mut format!("optimizedP: {optimizedP}, lengthP1: {lengthP1}, lengthP2: {lengthP2}", 
    //     optimizedP=optimizedP, lengthP1=velocity_p1, lengthP2=velocity_p2));
    if !optimizedP.is_nan() {
        p1.vector.direction.x = (p1.vector.direction.x - optimizedP) / velocity_p1;
        p1.vector.direction.y = (p1.vector.direction.y - optimizedP) / velocity_p1;
        p2.vector.direction.x = (p2.vector.direction.x + optimizedP) / velocity_p2;
        p2.vector.direction.y = (p2.vector.direction.y + optimizedP) / velocity_p2;
    }

    // p1.vector.direction.x = invert(p1.vector.direction.x);
    // p1.vector.direction.y = invert(p1.vector.direction.y);
    // p2.vector.direction.x = invert(p2.vector.direction.x);
    // p2.vector.direction.y = invert(p2.vector.direction.y);
    move_point(&mut p1.vector);
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
        for particle in POINTS.iter_mut() {
            check_colide_edge(particle);    
            move_point(&mut particle.vector);
            check_particle_colisions(particle);
        }
    }
}


fn draw_point(particle: &mut Particle, context: & web_sys::CanvasRenderingContext2d) {
    let vector = &particle.vector;
    context.begin_path();
    context.arc(vector.location.x, vector.location.y, particle.radius, 0.0, 360.0).ok();
    unsafe {
        if particle.id % 3 == 0 {
            context.set_fill_style(&JsValue::from_str("rgb(255, 0, 0)"));
        } else if particle.id % 3 == 1 {
            context.set_fill_style(&JsValue::from_str("rgb(0, 255, 0)"));
        } else if particle.id % 3 == 2 {
            context.set_fill_style(&JsValue::from_str("rgb(0, 0, 255)"));
        }
    }
    context.fill();

    context.begin_path();
    context.set_stroke_style(&JsValue::from_str("rgb(255, 255, 255)"));
    context.move_to(vector.location.x, vector.location.y);
    context.line_to(vector.location.x+(vector.direction.x*4.0), vector.location.y+(vector.direction.y*4.0));
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
