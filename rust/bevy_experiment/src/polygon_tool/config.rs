use bevy::ecs::prelude::*;

#[derive(Component)]
pub struct PolygonToolConfig {
    /// Radius of the sphere for ray intersection.
    pub earth_radius: f64,
    /// Number of interpolation segments per great circle arc.
    pub arc_segments: usize,
    /// Radius of the control point gizmo spheres.
    pub point_gizmo_radius: f32,
}

impl Default for PolygonToolConfig {
    fn default() -> Self {
        Self {
            earth_radius: 1.0,
            arc_segments: 64,
            point_gizmo_radius: 0.02,
        }
    }
}
