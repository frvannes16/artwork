use geo::algorithm::convex_hull;
use geo::Coordinate;
use log::{debug, info};
use nannou::color;
use nannou::prelude::*;

const MAX_RECORDABLE_FRAMES: u64 = 20_000;
const RECORD: bool = false;

fn main() {
    env_logger::init();
    nannou::app(model).update(update).simple_window(view).run();
}

struct Poly {
    polygon: Vec<Point2>,
    color: color::Hsl,
}

struct Model {
    polygons: Vec<Poly>,
    random_points: Vec<Point2>,
    center_points: Vec<Point2>,
}

fn model(app: &App) -> Model {
    // Define some randomly dispersed points.
    info!("Generating 400 random vertices");
    let (min_x, max_x) = (app.window_rect().left(), app.window_rect().right());
    let (min_y, max_y) = (app.window_rect().bottom(), app.window_rect().top());
    let mut vertices: Vec<Point2> = (0..400)
        .map(|_| {
            Point2::new(
                random_range::<f32>(min_x, max_x),
                random_range::<f32>(min_y, max_y),
            )
        })
        .collect();
    info!("Done with random vertices");

    let mut polygons: Vec<Poly> = Vec::new();
    let mut sample_points: Vec<Point2> = Vec::new();
    let default_poly_hue = random_range::<f32>(0.0, 1.0);

    // Iterate over a uniform grid of points.
    for x_idx in 1..=20 {
        for y_idx in 1..=20 {
            info!("uniform point {},{}", x_idx, y_idx);
            let x = min_x + (max_x - min_x) / 20.0 * (x_idx as f32);
            let y = min_y + (max_y - min_y) / 20.0 * (y_idx as f32);
            let c = Point2::new(x, y);
            debug!("center point: {}", c);
            sample_points.push(c);
            // Use point p as loose center of each mesh.
            // Find k-nearest neighbors to p. Uses naive algorithm
            // iterate through all points, and find 5 nearest
            let k: usize = 5;
            vertices.sort_by(|&a, &b| c.distance(a).partial_cmp(&c.distance(b)).unwrap());
            let nearest_k = vertices[0..k].to_vec();

            // Produce convex hull from those nearest neighbors.
            let mut points: Vec<Coordinate<f32>> = nearest_k
                .iter()
                .map(|&p| Coordinate { x: p.x, y: p.y })
                .collect();
            let convex_hull = convex_hull::quick_hull(&mut points[..]);
            let hull_points = convex_hull.into_points();
            let polygon: Vec<Point2> = hull_points
                .iter()
                .map(|&p| Point2::new(p.x(), p.y()))
                .collect();
            let color = color::hsl(
                default_poly_hue,
                0.7,
                random_range::<f32>(0.0, 1.0),
            );
            polygons.push(Poly { polygon, color });
        }
    }
    Model {
        polygons,
        random_points: vertices,
        center_points: sample_points,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);

    model.polygons.iter().for_each(|poly| {
        // only vary the luminescense.
        draw.polygon()
            .points(poly.polygon.clone())
            .color(poly.color);
    });

    // model.center_points.iter().for_each(|&p| {
    //     draw.ellipse().xy(p).radius(2.0).color(RED);
    // });

    // model.random_points.iter().for_each(|&p| {
    //     draw.ellipse().xy(p).radius(2.0).color(BLUE);
    // });

    draw.to_frame(app, &frame).unwrap();

    if RECORD && app.elapsed_frames() < MAX_RECORDABLE_FRAMES {
        // Capture the frame!
        let file_path = captured_frame_path(app, &frame);
        app.main_window().capture_frame(file_path);
    }
}

fn captured_frame_path(app: &App, frame: &Frame) -> std::path::PathBuf {
    // Create a path that we want to save this frame to.
    app.project_path()
        .expect("failed to locate `project_path`")
        // Capture all frames to a directory called `<path_to_project>/frames`.
        .join("frames")
        // Name each file after the number of the frame. Numbers must have 5 digts, padded with 0 at start.
        .join(format!("{:05}", frame.nth()))
        // The extension will be PNG. We also support tiff, bmp, gif, jpeg, webp and some others.
        .with_extension("png")
}
