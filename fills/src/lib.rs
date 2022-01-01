use nannou::geom::{Rect, Vec2, Tri, Point2};
use nannou::rand::{thread_rng, random_range};
use nannou::rand::prelude::SliceRandom;


pub fn evenly_distributed_grid(rect: &Rect, density: f32) -> Vec<Vec2> {
    // create grid of points inside of the rect based on a specific density.
    let x_interval_distance = 100.0 / density;
    let y_interval_distance = 100.0 / density;
    let x_intervals = (rect.w() / x_interval_distance).round() as i32;
    let y_intervals = (rect.h() / y_interval_distance).round() as i32;

    let points_in_rect: Vec<Vec2> = (0..=x_intervals)
        .map(|x| {
            let vec: Vec<Point2> = (0..=y_intervals)
                .map(|y| {
                    return Vec2::new(
                        rect.left() + (x as f32) * x_interval_distance,
                        rect.bottom() + (y as f32) * y_interval_distance,
                    );
                })
                .collect();
            return vec;
        })
        .flatten()
        .collect();
    return points_in_rect;
}

pub fn offset_point_randomly(point: &Vec2, offset_scale: f32) -> Vec2 {
        let new_x = point.x + random_range::<f32>(-1.0, 1.0) * offset_scale;
        let new_y = point.y + random_range::<f32>(-1.0, 1.0) * offset_scale;
        Vec2::new(new_x, new_y)
}

pub fn randomly_ordered_grid_of_points(rect: &Rect, density: f32) -> Vec<Vec2> {
    let mut point_grid = evenly_distributed_grid(rect, density);
    let mut rng = thread_rng();
    point_grid.shuffle(&mut rng);
    return point_grid;
}


pub fn subtriangles(rect: &Rect, levels: i32) -> Vec<Vec<Vec2>> {
    let mut subdivisions: Vec<Rect> = rect.subdivisions().into();
    for _ in 0..levels {
        subdivisions = subdivisions
            .iter()
            .flat_map(|sub| sub.subdivisions())
            .collect();
    }
    let triangles: Vec<Tri<[f32; 2]>> = subdivisions
        .iter()
        .flat_map(|sub| sub.triangles_iter())
        .collect();
    let triangle_points: Vec<Vec<Vec2>> = triangles
        .iter()
        .map(|&t| {
            let points: Vec<Vec2> = t
                .vertices()
                .chain(t.vertices())
                .map(|[x, y]| Vec2::new(x, y))
                .collect();
            let three_points = points[0..3].to_vec();
            return three_points;
        })
        .collect();

    return triangle_points;
}

#[cfg(test)]
mod tests {
    use crate::{Rect, Vec2};

    // These tests just ensure that the functions run without panicking. 
    // Admittedly, these are not great tests.

    #[test]
    fn test_evenly_distributed_grid(){
        let in_rect = Rect::from_xy_wh(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let _grid = crate::evenly_distributed_grid(&in_rect, 10.0);
    }

    #[test]
    fn test_random_grid() {
        let in_rect = Rect::from_xy_wh(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let _random_grid = crate::randomly_ordered_grid_of_points(&in_rect, 10.0);
    }

    #[test]
    fn subtriangles_works() {
        let in_rect = Rect::from_xy_wh(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let _triangles = crate::subtriangles(&in_rect, 3);        
    }
}
