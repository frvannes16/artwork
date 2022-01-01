use std::collections::HashSet;

use fills;

use log::debug;
use nannou::color::rgba8;
use nannou::{prelude::*, rand::prelude::SliceRandom, rand::thread_rng, rand::random_range};

mod paper;

const MAX_RECORDABLE_FRAMES: u64 = 1;
const PADDING: f32 = 15.0;
const MARGIN: f32 = 100.0;
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

const WEIGHTED_COLORS: [((u8, u8, u8), f32); 6] = [
    ((0x97, 0xBD, 0xC2), 0.05),
    ((0x45, 0x86, 0x8F), 0.4),
    ((0xD7, 0xF5, 0xE8), 0.15),
    ((0xFA, 0x7A, 0x7A), 0.2),
    ((0xC2, 0x97, 0xAC), 0.10),
    ((0x00, 0x00, 0x00), 0.10),
];


#[derive(Copy, Clone)]
enum FillType {
    Dots,
    Triangles,
    Solid,
    Mesh,
    Empty,
}

const WEIGHTED_FILL_TYPE: [(FillType, f32); 5] = [
    (FillType::Dots, 0.2),
    (FillType::Triangles, 0.2),
    (FillType::Solid, 0.2),
    (FillType::Mesh, 0.2),
    (FillType::Empty, 0.2),
];

const CHAIN_MIN: i32 = 4;
const CHAIN_MAX: i32 = 13;
const BACKGROUND: (u8, u8, u8) = (0xFD, 0xF9, 0xF5);

fn main() {
    env_logger::init();
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .exit(exit)
        .run();
}

fn model(app: &App) -> Model {
    // Let's write to A4 portrait page.
    // let paper = paper::Paper::from_iso216(paper::ISO216::A4, paper::Orientation::Portrait).unwrap();
    let dimensions: (u32, u32) = (1440, 2560);
    let texture_dimensions = [dimensions.0, dimensions.1];
    let pixels_per_cell = random_range::<u32>(5, 300);
    let (columns, rows) = (dimensions.0 / pixels_per_cell, dimensions.1 / pixels_per_cell);

    let [win_h, win_w] = [dimensions.0 / 4, dimensions.1 / 4];
    let w_id = app
        .new_window()
        .size(win_w, win_h)
        .title("nannou cells")
        .view(view)
        .build()
        .unwrap();
    let window = app.window(w_id).unwrap();

    // Retrieve the WGPU device
    let device = window.device();

    // Create our custom texture to capture large content.
    let sample_count = window.msaa_samples();
    let texture = wgpu::TextureBuilder::new()
        .size(texture_dimensions)
        // Our texture will be used as the RENDER_ATTACHMENT for our `Draw` render pass.
        // It will also be SAMPLED by the `TextureCapturer` and `TextureResizer`.
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
        // Use nannou's default multisampling sample count.
        .sample_count(sample_count)
        // Use a spacious 16-bit linear sRGBA format suitable for high quality drawing.
        .format(wgpu::TextureFormat::Rgba16Float)
        // Build it!
        .build(device);

    // Create our `Draw` instance and a renderer for it.
    let draw = nannou::Draw::new();
    let descriptor = texture.descriptor();
    let renderer =
        nannou::draw::RendererBuilder::new().build_from_texture_descriptor(device, descriptor);

    // Create the texture capturer.
    let texture_capturer = wgpu::TextureCapturer::default();

    // Create the texture reshaper.
    let texture_view = texture.view().build();
    let texture_sample_type = texture.sample_type();
    let dst_format = Frame::TEXTURE_FORMAT;
    let texture_reshaper = wgpu::TextureReshaper::new(
        device,
        &texture_view,
        sample_count,
        texture_sample_type,
        sample_count,
        dst_format,
    );

    // STARTING SHAPE BUILDING
    // Create a grid of cells.
    // While there are cells not filled in the grid, fill one at random and then
    // randomly pick a direction and occupy the cells in that direction for as long as
    // possible or until some random limit.
    // Repeat while loop.

    debug!("Creating grid");
    let grid = &mut Grid::new(columns, rows);
    let mut chains: Vec<Chain> = Vec::new();
    // First, pop off random number of empty cells.

    while grid.has_cells() {
        let chain_len = random_range::<i32>(CHAIN_MIN, CHAIN_MAX);
        let mut chain = Vec::new();
        let chain_direction = random_direction();

        let (mut x, mut y) = grid.peek_random().unwrap();

        debug!("Sampling cell ({},{})", x, y);
        if !grid.cell_taken(&(x, y)) {
            let starting_cell: Cell = grid.take_cell(&(x, y)).unwrap();
            debug!("Took cell {:?} START", starting_cell);
            chain.push(starting_cell);

            while let Some(next_cell_coordinates) =
                grid.adjacent_cell_coordinates(&(x, y), &chain_direction)
            {
                if let Some(next_cell) = grid.take_cell(&next_cell_coordinates) {
                    debug!("Took cell {:?}", next_cell);
                    chain.push((next_cell.0, next_cell.1));
                    x = next_cell.0;
                    y = next_cell.1;
                } else {
                    debug!("ending chain {}", chain.len());
                    break;
                }

                if (chain.len() as i32) > chain_len {
                    // end this chain while it's possible.
                    debug!("ending chain {}", chain.len());
                    break;
                }
            }

            chains.push(Chain::from_cells(chain));
        }
    }

    Model {
        texture,
        draw,
        renderer,
        texture_capturer,
        texture_reshaper,

        w: columns,
        h: rows,
        chains,
        margin: MARGIN,
        padding: PADDING,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // First reset draw state.
    let draw = &model.draw;
    draw.reset();

    // draw like we normally would in the view.
    let background = Rgb::new(BACKGROUND.0, BACKGROUND.1, BACKGROUND.2);
    draw.background().color(background);
    let texture_dimensions = model.texture.size();
    let window_rect = Rect::from_wh(Vec2::new(
        texture_dimensions[0] as f32,
        texture_dimensions[1] as f32,
    ));

    for Chain {
        cells,
        color,
        fill_type,
    } in &model.chains
    {
        if cells.len() > 0 {
            // Combine the cells into one rectangle.
            let chain_rect = build_chain_rect(&cells, model, &window_rect);

            // Fill the rectangle.
            match *fill_type {
                FillType::Empty => {},
                FillType::Dots => {
                    let density = random_range(5.0, 20.0);
                    let grid_of_points = fills::evenly_distributed_grid(&chain_rect, density);

                    // Offset all points by a random amount multiplied by offset_scale
                    let offset_scale: f32 = 5.0;
                    grid_of_points
                        .iter()
                        .map(|p| fills::offset_point_randomly(p, offset_scale))
                        .filter(|p| chain_rect.contains(*p))
                        .for_each(|p| {
                            draw.ellipse().radius(1.0).color(*color).xy(p);
                        });
                },
                FillType::Solid => {
                    draw.rect()
                        .wh(chain_rect.wh())
                        .xy(chain_rect.xy())
                        .color(*color);
                },
                FillType::Mesh => {
                    let density = random_range(5.0, 20.0);
                    let points = fills::randomly_ordered_grid_of_points(&chain_rect, density);
                    draw.polyline().color(*color).weight(1.0).points(points);
                },
                FillType::Triangles => {
                    let levels = random_range(2, 7);
                    let points = fills::subtriangles(&chain_rect, levels);
                    points.into_iter().for_each(|tri| {
                        draw.polyline().color(*color).weight(1.0).points(tri);
                    });
                },

            }

            draw.rect()
                .wh(chain_rect.wh())
                .xy(chain_rect.xy())
                .color(rgba8(0, 0, 0, 0))
                .stroke_color(*color)
                .stroke_weight(2.0);
        }
    }

    if app.elapsed_frames() < MAX_RECORDABLE_FRAMES {
        // Render our drawing to the texture.
        let window = app.main_window();
        let device = window.device();
        let ce_desc = wgpu::CommandEncoderDescriptor {
            label: Some("texture renderer"),
        };
        let mut encoder = device.create_command_encoder(&ce_desc);
        model
            .renderer
            .render_to_texture(device, &mut encoder, draw, &model.texture);

        // Take a snapshot of the texture. The capturer will do the following:
        //
        // 1. Resolve the texture to a non-multisampled texture if necessary.
        // 2. Convert the format to non-linear 8-bit sRGBA ready for image storage.
        // 3. Copy the result to a buffer ready to be mapped for reading.
        let snapshot = model
            .texture_capturer
            .capture(device, &mut encoder, &model.texture);

        // Submit the commands for our drawing and texture capture to the GPU.
        window.queue().submit(Some(encoder.finish()));

        // Submit a function for writing our snapshot to a PNG.
        //
        // NOTE: It is essential that the commands for capturing the snapshot are `submit`ted before we
        // attempt to read the snapshot - otherwise we will read a blank texture!
        let elapsed_frames = app.main_window().elapsed_frames();
        let path = capture_frame_directory(app)
            .join(elapsed_frames.to_string())
            .with_extension("png");
        snapshot
            .read(move |result| {
                let image = result.expect("failed to map texture memory").to_owned();
                image
                    .save(&path)
                    .expect("failed to save texture to png image");
            })
            .unwrap();
    }
}

fn view(_app: &App, model: &Model, frame: Frame) {
    // Sample the texture and write it to the frame.
    let mut encoder = frame.command_encoder();
    model
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut *encoder);
}

// Wait for capture to finish.
fn exit(app: &App, model: Model) {
    println!("Waiting for PNG writing to complete...");
    let window = app.main_window();
    let device = window.device();
    model
        .texture_capturer
        .await_active_snapshots(&device)
        .unwrap();
    println!("Done!");
}

fn build_chain_rect(chain: &Vec<Cell>, model: &Model, window: &Rect) -> Rect {
    // Takes a vec of cells and returns the dimensions and position of a rectangle
    // that wraps all of the cells and accounts for any padding and margin.
    let first_cell = chain.first().unwrap();
    let last_cell = chain.last().unwrap();
    // Height and width without padding.
    let cell_width = (window.w() - (model.margin * 2.0)) / (model.w as f32);
    let cell_height = (window.h() - (model.margin * 2.0)) / (model.h as f32);

    let rect_w =
        (((last_cell.0 - first_cell.0).abs() + 1).max(1) as f32) * cell_width - model.padding;
    let rect_h =
        (((last_cell.1 - first_cell.1).abs() + 1).max(1) as f32) * cell_height - model.padding;
    let rect_x = window.left()
        + model.margin
        + (last_cell.0.min(first_cell.0) as f32) * cell_width
        + rect_w / 2.0
        + model.padding;
    let rect_y = window.bottom()
        + model.margin
        + (last_cell.1.min(first_cell.1) as f32) * cell_height
        + rect_h / 2.0
        + model.padding;
    Rect::from_xy_wh(Point2::new(rect_x, rect_y), Vec2::new(rect_w, rect_h))
}

fn random_direction() -> Direction {
    let mut rng = thread_rng();

    let result = [
        (Direction::UP, 0.25),
        (Direction::DOWN, 0.25),
        (Direction::LEFT, 0.25),
        (Direction::RIGHT, 0.25),
    ]
    .choose_weighted(&mut rng, |dir| dir.1);
    result.unwrap().0
}

fn capture_frame_directory(app: &App) -> std::path::PathBuf {
    app.project_path()
        .expect("failed to locate `project_path`")
        // Capture all frames to a directory called `<path_to_project>/frames`.
        .join("frames")
}

struct Chain {
    cells: Vec<Cell>,
    color: Srgb<u8>,
    fill_type: FillType,
}

impl Chain {
    fn new(cells: Vec<Cell>, color: Srgb<u8>, fill_type: FillType) -> Self {
        Chain {
            cells,
            color,
            fill_type,
        }
    }

    fn from_cells(cells: Vec<Cell>) -> Self {
        let mut rng = thread_rng();
        // Randomly select the color and fill type for the rest of the chain.
        let color_bytes = WEIGHTED_COLORS
            .choose_weighted(&mut rng, |item| item.1)
            .unwrap()
            .0;
        let color = Rgb::new(color_bytes.0, color_bytes.1, color_bytes.2);

        let fill_type = WEIGHTED_FILL_TYPE
            .choose_weighted(&mut rng, |item| item.1)
            .unwrap()
            .0;
        Chain::new(cells, color, fill_type)
    }
}

struct Model {
    // The texture that we will draw to.
    texture: wgpu::Texture,
    // Create a `Draw` instance for drawing to our texture.
    draw: nannou::Draw,
    // The type used to render the `Draw` vertices to our texture.
    renderer: nannou::draw::Renderer,
    // The type used to capture the texture.
    texture_capturer: wgpu::TextureCapturer,
    // The type used to resize our texture to the window texture.
    texture_reshaper: wgpu::TextureReshaper,
    // Art fields BELOW
    w: u32,
    h: u32,
    margin: f32,
    padding: f32,
    chains: Vec<Chain>,
}

#[derive(Clone, Copy)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

type Cell = (i32, i32);

struct Grid {
    available_cells: HashSet<Cell>,
    w: u32,
    h: u32,
}

impl Grid {
    fn new(w: u32, h: u32) -> Self {
        let available_cells: HashSet<Cell> = (0..w)
            .map(|i| return (0..h).map(|j| (i as i32, j as i32)).collect::<Vec<Cell>>())
            .flatten()
            .collect();
        Grid {
            available_cells,
            w,
            h,
        }
    }

    fn peek_random(&self) -> Option<Cell> {
        let v: Vec<Cell> = self.available_cells.iter().map(|&(a, b)| (a, b)).collect();
        let mut rng = thread_rng();
        let r = v.choose(&mut rng)?;
        Some((r.0, r.1))
    }

    fn cell_taken(&self, cell: &Cell) -> bool {
        return !self.available_cells.contains(cell);
    }

    fn take_cell(&mut self, cell: &Cell) -> Option<Cell> {
        if self.cell_taken(cell) {
            None
        } else {
            self.available_cells.take(cell)
        }
    }

    fn has_cells(&self) -> bool {
        return self.available_cells.len() > 0;
    }

    fn adjacent_cell_coordinates(&self, cell: &Cell, direction: &Direction) -> Option<Cell> {
        let icell = (cell.0 as i32, cell.1 as i32);
        let diff: (i32, i32) = match direction {
            Direction::UP => (0, 1),
            Direction::DOWN => (0, -1),
            Direction::LEFT => (-1, 0),
            Direction::RIGHT => (1, 0),
        };
        let adj = (icell.0 + diff.0, icell.1 + diff.1);
        if adj.0 < 0 || adj.0 >= self.w as i32 || adj.1 < 0 || adj.1 >= self.h as i32 {
            None
        } else {
            return Some((adj.0, adj.1));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_available_cells() {
        let grid = crate::Grid::new(20, 20);
        let sample = (1, 1);
        assert_eq!(grid.cell_taken(&sample), false)
    }

    #[test]
    fn take_cell() {
        let mut grid = crate::Grid::new(20, 20);
        let sample = (1, 1);
        let grid_size = grid.available_cells.len();
        assert_eq!(grid.cell_taken(&sample), false);
        assert_eq!(grid.take_cell(&sample).unwrap(), (1, 1));
        assert_eq!(grid.available_cells.len(), grid_size - 1);
        assert_eq!(grid.cell_taken(&sample), true);
    }

    #[test]
    fn find_adjacent_cells() {
        let grid = crate::Grid::new(10, 10);
        let start = (9, 9);
        let next = grid.adjacent_cell_coordinates(&start, &crate::Direction::DOWN);
        assert_eq!(next, Some((9, 8)));
        let out_of_bounds = grid.adjacent_cell_coordinates(&start, &crate::Direction::UP);
        assert_eq!(out_of_bounds, None);

        let left = grid.adjacent_cell_coordinates(&start, &crate::Direction::LEFT);
        assert_eq!(left, Some((8, 9)));

        let right_out_of_bounds = grid.adjacent_cell_coordinates(&start, &crate::Direction::RIGHT);
        assert_eq!(right_out_of_bounds, None);
    }

    #[test]
    fn oob_adjacent_cells_from_origin() {
        let grid = crate::Grid::new(10, 10);
        let start = (0, 0);
        let down = grid.adjacent_cell_coordinates(&start, &crate::Direction::DOWN);
        assert_eq!(down, None);

        let left = grid.adjacent_cell_coordinates(&start, &crate::Direction::LEFT);
        assert_eq!(left, None);
    }

    #[test]
    fn test_take_random_cells() {
        let mut grid = crate::Grid::new(10, 10);
        let mut count_taken = 0;

        while let Some(cell) = grid.peek_random() {
            if let Some(taken) = grid.take_cell(&cell) {
                assert_eq!(taken, cell);
                count_taken = count_taken + 1;
            } else {
                break;
            }
        }
        assert_eq!(count_taken, 10 * 10);
    }

    #[test]
    fn long_chain_cell_fetching() {
        let mut grid = crate::Grid::new(1, 5);
        // check that we can create long chains using the while loop logic in the update fn.
        let chain_len = 5;
        while grid.has_cells() {
            let mut chain = Vec::new();
            let chain_direction = crate::Direction::RIGHT;

            let (mut x, mut y) = (0, 0);

            while !grid.cell_taken(&(x, y)) {
                // take cell
                let cell: crate::Cell = grid.take_cell(&(x, y)).unwrap();
                chain.push(cell);
                // Get next cell coordinates.
                match grid.adjacent_cell_coordinates(&(cell.0, cell.1), &chain_direction) {
                    Some((new_x, new_y)) => {
                        x = new_x;
                        y = new_y;
                    }
                    None => break, //  end this chain.
                }
                if chain.len() > chain_len {
                    break;
                }
            }

            if !grid.cell_taken(&(x, y)) {
                let starting_cell: crate::Cell = grid.take_cell(&(x, y)).unwrap();
                chain.push(starting_cell);

                while let Some(next_cell_coordinates) =
                    grid.adjacent_cell_coordinates(&(x, y), &chain_direction)
                {
                    if let Some(next_cell) = grid.take_cell(&next_cell_coordinates) {
                        chain.push((next_cell.0, next_cell.1));
                        x = next_cell.0;
                        y = next_cell.1;
                    } else {
                        break;
                    }

                    if chain.len() < chain_len as usize {
                        // end this chain while it's possible.
                        break;
                    }
                }
            }
            assert_eq!(chain.len(), 5);
        }
    }
}
