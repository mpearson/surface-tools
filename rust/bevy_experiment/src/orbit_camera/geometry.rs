use bevy::{
    math::{prelude::*, DVec3},
    prelude::Camera,
    transform::components::GlobalTransform,
};

/// Ray-sphere intersection test with f64 precision.
/// Returns the distance along the ray to the first intersection point, if any.
pub(super) fn ray_sphere_intersection_f64(
    ray_origin: DVec3,
    ray_direction: DVec3,  // Must be normalized
    sphere_center: DVec3,
    sphere_radius: f64,
    max_distance: f64,
) -> Option<f64> {
    let offset = ray_origin - sphere_center;
    let projected = offset.dot(ray_direction);
    let closest_point = offset - projected * ray_direction;
    let distance_squared = sphere_radius * sphere_radius - closest_point.length_squared();

    if distance_squared < 0.0
        || projected.copysign(-projected).powi(2) < -distance_squared
    {
        None
    } else {
        let toi = -projected - distance_squared.sqrt();
        if toi > max_distance {
            None
        } else {
            Some(toi.max(0.0))
        }
    }
}

/// Convert cursor position to world coordinates on a sphere using f64 precision.
pub(super) fn cursor_to_world_on_sphere_f64(
    cursor: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    sphere_radius: f64,
) -> Option<DVec3> {
    let viewport_pos = Vec2::new(cursor.x, cursor.y);
    let ray = camera
        .viewport_to_world(camera_transform, viewport_pos)
        .ok()?;

    // Convert f32 ray to f64 for higher precision
    let ray_origin = ray.origin.as_dvec3();
    let ray_direction = ray.direction.as_dvec3().normalize();

    let distance = ray_sphere_intersection_f64(
        ray_origin,
        ray_direction,
        DVec3::ZERO,
        sphere_radius,
        f64::MAX,
    )?;

    let intersection = ray_origin + ray_direction * distance;
    Some(intersection)
}

/// Calculate latitude from a world-space position
pub(super) fn compute_latitude(position: DVec3) -> f64 {
    position.y.atan2(position.xy().length())
}

/// Calculate longitude/azimuthal angle from a world-space position
pub(super) fn compute_longitude(position: DVec3) -> f64 {
    position.z.atan2(position.x)
}
