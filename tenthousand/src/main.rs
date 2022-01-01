use nannou::color::rgb_u32;
use nannou::noise::{NoiseFn, Perlin};
use nannou::prelude::*;
use std::f64::consts::PI;

const MAX_RECORDABLE_FRAMES: u64 = 1;

// const WEIGHTED_COLORS: [((u8, u8, u8), f32); 5] = [
//     ((0x4C, 0xBF, 0xC7), 0.2),
//     ((0x82, 0x93, 0x94), 0.2),
//     ((0x78, 0xFA, 0xBA), 0.2),
//     ((0xFB, 0xB7, 0xBF), 0.2),
//     ((0xC7, 0x4C, 0x98), 0.2),
// ];

// const WEIGHTED_COLORS: [((u8, u8, u8), f32); 5] = [
//     ((0x4C, 0xBF, 0xC7), 0.05),
//     ((0x82, 0x93, 0x94), 0.6),
//     ((0x78, 0xFA, 0xBA), 0.15),
//     ((0xC7, 0x4C, 0x98), 0.2),
//     ((0xFF, 0xFF, 0xFF), 0.00),
// ];

// const WEIGHTED_COLORS: [((u8, u8, u8), f32); 5] = [
//     ((0x97, 0xBD, 0xC2), 0.05),
//     ((0x45, 0x86, 0x8F), 0.6),
//     ((0xD7, 0xF5, 0xE8), 0.15),
//     ((0xFA, 0x7A, 0x7A), 0.2),
//     ((0xC2, 0x97, 0xAC), 0.00),
// ];

const BACKGROUND: u32 = 0xFDF9F5;

fn main() {
    env_logger::init();
    nannou::app(model).update(update).simple_window(view).run();
}

type Triangle = geom::Tri<[f32; 2]>;

struct Model {
    // Art fields BELOW
    triangles: Vec<Triangle>,
    noise: Perlin,
}

fn model(app: &App) -> Model {
    let window = app.window_rect();
    let bigger_rect = Rect::from_xy_wh(window.xy(), Vec2::new(window.w() * 3f32, window.h() * 3f32));
    
    // create subdivisions from window rect.
    let triangles: Vec<Triangle> = subtriangles(&bigger_rect, 7);

    // use a reusable perlin noise map which update() will move the triangles over.
    let perlin = Perlin::new();

    Model {
        triangles,
        noise: perlin,
    }
}

fn subtriangles(rect: &Rect, levels: i32) -> Vec<Triangle> {
    let mut subdivisions: Vec<Rect> = rect.subdivisions().into();
    for _ in 0..levels {
        subdivisions = subdivisions
            .iter()
            .flat_map(|sub| sub.subdivisions())
            .collect();
    }
    let triangles: Vec<Triangle> = subdivisions
        .iter()
        .flat_map(|sub| sub.triangles_iter())
        .collect();
    return triangles;
}

fn convert_ratio_to_heading(ratio: f64) -> Vec2 {
    let radians = ratio * 2f64 * PI;
    let heading_vector = Vec2::new(radians.cos() as f32, radians.sin() as f32);
    heading_vector
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // shift every point in the angle of the noise.
    model.triangles = model
        .triangles
        .iter()
        .map(|tri| {
            let f32_verts: [[f32; 2]; 3] = tri.0;
            let verts: [[f64; 2]; 3] = [
                [f32_verts[0][0] as f64, f32_verts[0][1] as f64],
                [f32_verts[1][0] as f64, f32_verts[1][1] as f64],
                [f32_verts[2][0] as f64, f32_verts[2][1] as f64],
            ];
            let heading_a = convert_ratio_to_heading(model.noise.get([verts[0][0] * 0.001f64, verts[0][1] * 0.001f64, 0.04213f64]));
            let heading_b = convert_ratio_to_heading(model.noise.get([verts[1][0] * 0.001f64, verts[1][1] * 0.001f64, 0.04213f64]));
            let heading_c = convert_ratio_to_heading(model.noise.get([verts[2][0] * 0.001f64, verts[2][1] * 0.001f64, 0.04213f64]));
            return geom::Tri::from_index_tri(&[
                [
                    f32_verts[0][0] + heading_a.x as f32,
                    f32_verts[0][1] + heading_a.y as f32,
                ],
                [
                    f32_verts[1][0] + heading_b.x as f32,
                    f32_verts[1][1] + heading_b.y as f32,
                ],
                [
                    f32_verts[2][0] + heading_c.x as f32,
                    f32_verts[2][1] + heading_c.y as f32,
                ],
            ], &[0,1,2]);
        })
        .collect();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(rgb_u32(BACKGROUND));

    let triangles = &model.triangles;
    triangles.into_iter().for_each(|tri| {
        draw.polyline()
            .color(BLACK)
            .weight(0.5f32)
            .points(tri.to_vec());
    });

    draw.to_frame(app, &frame).unwrap();

    if app.elapsed_frames() < MAX_RECORDABLE_FRAMES {
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
