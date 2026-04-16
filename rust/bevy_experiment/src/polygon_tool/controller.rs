use bevy::{
    color::Color,
    ecs::prelude::*,
    gizmos::prelude::Gizmos,
    math::{prelude::*, DQuat, DVec3},
    prelude::{Camera, Camera3d},
    transform::components::GlobalTransform,
};

use crate::common::rotation::great_circle_rotation;
use crate::orbit_camera::geometry::cursor_to_world_on_sphere_f64;

use super::config::PolygonToolConfig;
use super::events::PolygonToolInputEvent;
use super::state::PolygonToolState;

fn draw_great_circle_arc(
    gizmos: &mut Gizmos,
    from: DVec3,
    to: DVec3,
    segments: usize,
    color: Color,
) {
    let rotation = great_circle_rotation(from, to);
    let points: Vec<Vec3> = (0..=segments)
        .map(|i| {
            let t = i as f64 / segments as f64;
            (DQuat::IDENTITY.slerp(rotation, t) * from).as_vec3()
        })
        .collect();
    gizmos.linestrip(points, color);
}

pub fn step(
    mut gizmos: Gizmos,
    mut input_reader: MessageReader<PolygonToolInputEvent>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut tools: Query<(&PolygonToolConfig, &mut PolygonToolState)>,
) {
    let Some(input) = input_reader.read().next() else {
        return;
    };

    let Some((camera, camera_transform)) = cameras.iter().next() else {
        return;
    };

    for (config, mut state) in &mut tools {
        if let Some(screen_pos) = input.left_click {
            if let Some(world_pos) = cursor_to_world_on_sphere_f64(
                screen_pos,
                camera,
                camera_transform,
                config.earth_radius,
            ) {
                state.control_points.push(world_pos);
            }
        }
        if input.right_click {
            state.control_points.pop();
        }

        let green = Color::srgb(0.0, 1.0, 0.0);

        for point in &state.control_points {
            gizmos.sphere(
                Isometry3d::from_translation(point.as_vec3()),
                config.point_gizmo_radius,
                green,
            );
        }

        for pair in state.control_points.windows(2) {
            draw_great_circle_arc(&mut gizmos, pair[0], pair[1], config.arc_segments, green);
        }
    }
}
