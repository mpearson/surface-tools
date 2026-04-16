use bevy::{ecs::prelude::*, math::f64::DVec3};

#[derive(Component, Default)]
pub struct PolygonToolState {
    /// Control points on the sphere surface, in world space (f64).
    pub control_points: Vec<DVec3>,
}
