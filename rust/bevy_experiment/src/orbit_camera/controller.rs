use bevy::{
    color::Color,
    ecs::prelude::*,
    gizmos::prelude::Gizmos,
    math::{
        bounding::{BoundingSphere, RayCast3d},
        prelude::*,
        DQuat, DVec2, DVec3,
    },
    prelude::Camera,
    time::Time,
    transform::components::{GlobalTransform, Transform},
};

use crate::orbit_camera::{
    events::OrbitCameraInputEvent,
    state::{OrbitCameraState, PanState},
};

use super::config::OrbitCameraConfig;

const POSITION_EPSILON: f64 = 1e-4;

fn distance_to_zoom_level(distance: f64) -> f64 {
    -distance.ln()
}

fn zoom_level_to_distance(zoom_level: f64) -> f64 {
    (-zoom_level).exp()
}

fn initialize_zoom_state(state: &mut OrbitCameraState) {
    if state.current_zoom_level == 0.0 && state.zoom_level_target == 0.0 {
        let radius = state.radius.max(f64::EPSILON);
        let zoom_level = distance_to_zoom_level(radius);
        state.current_zoom_level = zoom_level;
        state.zoom_level_target = zoom_level;
    }
}

fn update_zoom(config: &OrbitCameraConfig, state: &mut OrbitCameraState, zoom_delta: f64, dt: f32) {
    initialize_zoom_state(state);

    if zoom_delta != 0.0 {
        state.zoom_level_target -= zoom_delta;
    }

    let min_zoom_level = distance_to_zoom_level(config.max_distance);
    let max_zoom_level = distance_to_zoom_level(config.min_distance);
    state.zoom_level_target = state
        .zoom_level_target
        .clamp(min_zoom_level, max_zoom_level);

    let smoothing = (config.zoom_smoothing * dt as f64).min(1.0);
    if smoothing > 0.0 {
        state.current_zoom_level +=
            (state.zoom_level_target - state.current_zoom_level) * smoothing;
    } else {
        state.current_zoom_level = state.zoom_level_target;
    }

    state.radius = zoom_level_to_distance(state.current_zoom_level);
}

fn update_orbit(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    orbit_delta: Option<Vec2>,
    dt: f32,
) {
    if let Some(delta) = orbit_delta {
        state.euler_angles_target_delta.x -= delta.y;
        state.euler_angles_target_delta.y += delta.x;
    }

    let smoothing = config.orbit_smoothing * dt;
    let euler_step = if smoothing > 0.0 {
        let delta = state.euler_angles_target_delta * smoothing;
        state.euler_angles_target_delta -= delta;
        delta
    } else {
        let delta = state.euler_angles_target_delta;
        state.euler_angles_target_delta = Vec3::ZERO;
        delta
    };

    state.current_euler_angles += euler_step;

    if state.current_euler_angles.x > 180.0 {
        state.current_euler_angles.x -= 360.0;
    } else if state.current_euler_angles.x < -180.0 {
        state.current_euler_angles.x += 360.0;
    } else if state.current_euler_angles.x > 90.0 {
        state.current_euler_angles.x = 0.0;
    }

    state.current_euler_angles.x = state
        .current_euler_angles
        .x
        .clamp(config.min_theta, config.max_theta);

    state.current_euler_angles.y = state.current_euler_angles.y.rem_euclid(360.0);
    state.current_euler_angles.z = 0.0;
}

// fn cursor_to_world_on_plane(
//     cursor: Vec2,
//     camera: &Camera,
//     camera_transform: &GlobalTransform,
//     plane_height: f32,
// ) -> Option<DVec3> {
//     let viewport_pos = Vec2::new(cursor.x, cursor.y);
//     let ray = camera
//         .viewport_to_world(camera_transform, viewport_pos)
//         .ok()?;
//     let plane_origin = Vec3::new(0.0, plane_height, 0.0);
//     let plane = InfinitePlane3d::new(Vec3::Y);

//     let Some(distance) = ray.intersect_plane(plane_origin, plane) else {
//         return None;
//     };

//     let intersection = ray.get_point(distance);
//     Some(DVec3::new(
//         intersection.x as f64,
//         plane_height as f64,
//         intersection.z as f64,
//     ))
// }

fn cursor_to_world_on_sphere(
    cursor: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    // sphere_center: Vec3,
    sphere_radius: f32,
) -> Option<DVec3> {
    let viewport_pos = Vec2::new(cursor.x, cursor.y);
    let ray = camera
        .viewport_to_world(camera_transform, viewport_pos)
        .ok()?;
    let ray_cast = RayCast3d::from_ray(ray, f32::MAX);
    // let sphere = BoundingSphere::new(sphere_center, sphere_radius);
    let sphere = BoundingSphere::new(Vec3::ZERO, sphere_radius);

    let distance = ray_cast.sphere_intersection_at(&sphere)?;

    let intersection = ray.get_point(distance);
    Some(DVec3::new(
        intersection.x as f64,
        intersection.y as f64,
        intersection.z as f64,
    ))
}

fn update_pan_targets(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    input: &OrbitCameraInputEvent,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    if let Some(pan_delta) = input.pan_delta {
        if let Some(pan_start_screen_space) = input.pan_start_screen_space {
            // cursor_to_world_on_plane(pan_start_screen_space, camera, camera_transform, 0.0)
            if let Some(pan_start_world_space) = cursor_to_world_on_sphere(
                pan_start_screen_space,
                camera,
                camera_transform,
                config.earth_radius,
            ) {
                let start_world_space = DVec3::from(pan_start_world_space);
                state.pan = Some(PanState {
                    start_screen_space: pan_start_screen_space,
                    offset_screen_space: Vec2::ZERO,
                    start_world_space,
                    start_radius: start_world_space.length(),
                    current_world_space: start_world_space,
                    start_camera_transform: camera_transform.clone(),
                });
            } else {
                state.pan = None;
            }
        } else if let Some(pan_state) = state.pan.as_mut() {
            // Already panning, so just update the scren-space offset with the latest delta.
            // state.pan_cursor_position += Vec2::new(pan_delta.x, pan_delta.y);
            pan_state.offset_screen_space += Vec2::new(pan_delta.x, pan_delta.y);
        }

        if let Some(pan_state) = state.pan.as_mut() {
            // if let Some(mouse_pos_world_space) = cursor_to_world_on_plane(
            //     pan_state.start_screen_space + pan_state.offset_screen_space,
            //     camera,
            //     camera_transform,
            //     0.0,
            // ) {
            if let Some(mouse_pos_world_space) = cursor_to_world_on_sphere(
                pan_state.start_screen_space + pan_state.offset_screen_space,
                camera,
                &pan_state.start_camera_transform,
                pan_state.start_radius as f32,
            ) {
                pan_state.current_world_space = mouse_pos_world_space;

                state.pan_rotation_target = DQuat::from_rotation_arc(
                    mouse_pos_world_space.normalize(),
                    pan_state.start_world_space.normalize(),
                );
            }
        }
    } else {
        state.pan = None;
    }
}

fn update_position(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    camera_transform: &mut Transform,
    gizmos: &mut Gizmos,
    dt: f32,
) {
    gizmos.sphere(Vec3::ZERO, config.earth_radius as f32, Color::WHITE);
    // Debug gizmos for pan positions
    if let Some(pan_state) = &state.pan {
        gizmos.sphere(
            Isometry3d::from_translation(pan_state.start_world_space.as_vec3()),
            0.1,
            Color::srgb(1.0, 0.0, 0.0),
        );
        gizmos.sphere(
            Isometry3d::from_translation(pan_state.current_world_space.as_vec3()),
            0.1,
            Color::srgb(0.0, 1.0, 0.0),
        );
    }

    let smoothing = (config.pan_smoothing * dt as f64).min(1.0);
    let radius = state.radius.max(f64::EPSILON);

    // if smoothing > 0.0 {

    // if let Some(pan_state) = &state.pan {
    if state.pan.is_some() {
        let current_pan_rotation = DQuat::from_rotation_arc(
            DVec3::X,
            DVec3::from(camera_transform.translation / radius as f32),
        );
        // camera_transform.translation = (DVec3::from(camera_transform.translation))
        //     .rotate_towards(pan_state.current_world_space, 0.01)
        //     .as_vec3();

        camera_transform.translation =
            ((DQuat::slerp(current_pan_rotation, state.pan_rotation_target, smoothing) * DVec3::X)
                * radius)
                .as_vec3();
    }
    // }

    // state.pan_rotation_target = -DQuat::from_rotation_arc(
    //     pan_state.start_world_space.normalize(),
    //     mouse_pos_world_space.normalize(),
    // );

    // let pan_rotation_target = state.pan_rotation_target * current_pan_rotation;

    // camera_transform.translation =
    //     ((DQuat::slerp(pan_rotation_target, current_pan_rotation, smoothing) * DVec3::X) * radius)
    //         .as_vec3();
    // } else {
    // camera_transform.translation = camera_transform.translation.normalize() * radius as f32;
    // camera_transform.translation = Vec3::X * -radius as f32;
    // }
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);
}

pub fn step(
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut input_reader: MessageReader<OrbitCameraInputEvent>,
    mut cameras: Query<(
        &OrbitCameraConfig,
        &mut OrbitCameraState,
        &Camera,
        &GlobalTransform,
        &mut Transform,
    )>,
) {
    let Some(input) = input_reader.read().next() else {
        return;
    };

    let frame_dt = time.delta_secs().min(0.02);

    for (config, mut state, camera, camera_transform, mut transform) in &mut cameras {
        update_pan_targets(config, &mut state, &input, camera, camera_transform);
        update_zoom(config, &mut state, input.zoom_delta, frame_dt);
        update_orbit(config, &mut state, input.orbit_delta, frame_dt);
        update_position(config, &mut state, &mut transform, &mut gizmos, frame_dt);
    }
}
