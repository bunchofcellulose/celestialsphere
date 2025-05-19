use crate::circle::GreatCircle;
use crate::point::Point;
use crate::Vec3;

pub fn snap_to_great_circle(
    point: Vec3,
    great_circles: &[GreatCircle],
    points: &[Point],
    threshold: f64,
) -> Vec3 {
    let mut closest_distance = threshold;
    let mut snapped_point = point;

    // Check each great circle
    for gc in great_circles {
        let pole = points[gc.pole].rotated;

        // Distance from point to great circle plane is |dot(point, pole)|
        let distance = (point[0] * pole[0] + point[1] * pole[1] + point[2] * pole[2]).abs();

        if distance < closest_distance {
            // Project point onto the plane of the circle (subtract the component along the pole)
            let dot_product = point[0] * pole[0] + point[1] * pole[1] + point[2] * pole[2];
            let projected = [
                point[0] - dot_product * pole[0],
                point[1] - dot_product * pole[1],
                point[2] - dot_product * pole[2],
            ];

            // Normalize to get a point on the sphere
            let mag = (projected[0].powi(2) + projected[1].powi(2) + projected[2].powi(2)).sqrt();
            if mag > 1e-10 {
                snapped_point = [projected[0] / mag, projected[1] / mag, projected[2] / mag];
                closest_distance = distance;
            }
        }
    }

    snapped_point
}
