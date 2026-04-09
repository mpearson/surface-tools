use bevy::{
    color::Color,
    ecs::prelude::*,
    gizmos::prelude::Gizmos,
    math::{prelude::*, DQuat, DVec3},
    prelude::info,
    prelude::Camera,
    time::Time,
    transform::components::{GlobalTransform, Transform},
};

use crate::common::rotation::great_circle_rotation;
use crate::orbit_camera::{
    events::OrbitCameraInputEvent,
    plugin::{OrbitCameraChildRef, OrbitCameraRig},
    state::{OrbitCameraState, PanState, ZoomState},
};

use super::config::OrbitCameraConfig;
use super::geometry::{
    compute_latitude, compute_longitude, cursor_to_world_on_sphere_f64, nearest_point_on_sphere_f64,
};

fn distance_to_zoom_level(distance: f64) -> f64 {
    -distance.ln()
}

fn zoom_level_to_distance(zoom_level: f64) -> f64 {
    (-zoom_level).exp()
}

fn initialize_zoom_state(state: &mut OrbitCameraState) {
    if state.current_zoom_level == 0.0 && state.zoom_level_ref == 0.0 {
        let radius = state.radius.max(f64::EPSILON);
        let zoom_level = distance_to_zoom_level(radius);
        state.current_zoom_level = zoom_level;
        state.zoom_level_ref = zoom_level;
    }
}

fn update_zoom(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    input: &OrbitCameraInputEvent,
    cursor_position_world_space: &Option<Vec3>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    dt: f32,
    gizmos: &mut Gizmos,
) {
    initialize_zoom_state(state);

    // Handle zoom state initialization/updates
    if input.zoom_delta != 0.0 {
        if let Some(cursor_pos) = input.cursor_position {
            // Starting a new zoom operation - capture the world position under the cursor
            if cursor_position_world_space.is_some() {
                state.zoom = Some(ZoomState {
                    start_cursor_screen_space: cursor_pos,
                    start_world_space: cursor_position_world_space.unwrap().as_dvec3(),
                    start_radius: state.radius,
                });
            } else {
                state.zoom = None;
            }
        }

        state.zoom_level_ref -= input.zoom_delta;
    }

    // Clamp zoom level
    let min_zoom_level = distance_to_zoom_level(config.max_distance);
    let max_zoom_level = distance_to_zoom_level(config.min_distance);
    state.zoom_level_ref = state.zoom_level_ref.clamp(min_zoom_level, max_zoom_level);

    // Smooth interpolation of zoom level
    let smoothing = 1.0 - (-config.zoom_smoothing * dt as f64).exp();
    if smoothing > 0.0 {
        state.current_zoom_level += (state.zoom_level_ref - state.current_zoom_level) * smoothing;
    } else {
        state.current_zoom_level = state.zoom_level_ref;
    }

    state.radius = zoom_level_to_distance(state.current_zoom_level);

    // Clear zoom state when we're close to the target zoom level
    let zoom_threshold = 0.01;
    if (state.current_zoom_level - state.zoom_level_ref).abs() < zoom_threshold {
        state.zoom = None;
        state.zoom_rotation_reference = DQuat::IDENTITY;
    }

    // Calculate zoom rotation correction to keep the world point under the cursor
    if let Some(zoom_state) = &state.zoom {
        if let Some(rotation) = calculate_rotation_to_preserve_point(
            zoom_state.start_world_space,
            zoom_state.start_cursor_screen_space,
            camera,
            camera_transform,
            config.earth_radius,
        ) {
            state.zoom_rotation_reference = rotation;

            // Debug gizmos
            if let Some(current_world_pos) = cursor_to_world_on_sphere_f64(
                zoom_state.start_cursor_screen_space,
                camera,
                camera_transform,
                config.earth_radius,
            )
            .map(|v| v.as_vec3())
            {
                gizmos.sphere(
                    Isometry3d::from_translation(current_world_pos),
                    0.05,
                    Color::srgb(0.0, 0.0, 1.0), // Blue for current
                );
            }
            gizmos.sphere(
                Isometry3d::from_translation(zoom_state.start_world_space.as_vec3()),
                0.05,
                Color::srgb(1.0, 1.0, 0.0), // Yellow for start
            );
        }
    } else {
        // No active zoom, reset rotation target
        state.zoom_rotation_reference = DQuat::IDENTITY;
    }
}

fn update_orbit(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    orbit_delta: Option<Vec2>,
    dt: f32,
) {
    if let Some(delta) = orbit_delta {
        state.euler_angles_target_delta.x += delta.y;
        state.euler_angles_target_delta.y += delta.x;
    }

    let smoothing = 1.0 - (-config.orbit_smoothing * dt).exp();
    let euler_step = if config.orbit_smoothing > 0.0 {
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
    }

    state.current_euler_angles.x = state
        .current_euler_angles
        .x
        .clamp(config.min_pitch, config.max_pitch);

    state.current_euler_angles.y = state.current_euler_angles.y.rem_euclid(360.0);
    state.current_euler_angles.z = 0.0;
}

/// Calculates the rotation needed to keep a world point under the cursor constant.
/// Used by both pan and zoom to preserve cursor position during camera transformations.
fn calculate_rotation_to_preserve_point(
    start_world_pos: DVec3,
    current_cursor_pos: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    sphere_radius: f64,
) -> Option<DQuat> {
    let current_world_pos =
        cursor_to_world_on_sphere_f64(current_cursor_pos, camera, camera_transform, sphere_radius)?;

    return Some(great_circle_rotation(current_world_pos, start_world_pos));
}

// /// Remove roll component from a rotation quaternion using swing-twist decomposition.
// /// Decomposes the rotation into a swing (rotation perpendicular to radial axis) and
// /// twist (rotation around radial axis), then returns only the swing component.
// fn remove_roll_from_rotation(rotation: DQuat, camera_position: DVec3) -> DQuat {
//     let radial = camera_position.normalize();

//     // Swing-twist decomposition around the radial axis
//     // Given quaternion q = [x, y, z, w] and axis v, we decompose q = swing * twist
//     // where twist is rotation around v and swing is perpendicular to v

//     // Project quaternion's vector part onto the radial axis
//     let q_vec = DVec3::new(rotation.x, rotation.y, rotation.z);
//     let q_w = rotation.w;

//     let dot = q_vec.dot(radial);

//     // Twist quaternion (rotation around radial axis)
//     let twist = DQuat::from_xyzw(radial.x * dot, radial.y * dot, radial.z * dot, q_w);

//     let twist_len_sq =
//         twist.x * twist.x + twist.y * twist.y + twist.z * twist.z + twist.w * twist.w;

//     if twist_len_sq < 1e-10 {
//         // Degenerate case
//         return rotation;
//     }

//     let twist = twist.normalize();

//     // Swing quaternion (rotation perpendicular to radial axis)
//     // swing = q * twist^-1
//     let swing = rotation * twist.conjugate();

//     swing
// }

// /// Calculate how much to constrain roll based on latitude.
// /// Returns 1.0 at equator (full constraint), 0.0 at poles (no constraint)
// fn compute_roll_constraint_factor(
//     latitude_deg: f64,
//     low_threshold: f64,
//     high_threshold: f64,
// ) -> f64 {
//     let abs_lat = latitude_deg.abs();

//     if abs_lat <= low_threshold {
//         1.0 // Full constraint
//     } else if abs_lat >= high_threshold {
//         0.0 // No constraint
//     } else {
//         // Smooth transition using smoothstep
//         let t = (abs_lat - low_threshold) / (high_threshold - low_threshold);
//         let smooth_t = 3.0 * t * t - 2.0 * t * t * t;
//         1.0 - smooth_t
//     }
// }

// /// Apply latitude-based roll constraint to a rotation quaternion
// fn apply_roll_constraint(
//     rotation: DQuat,
//     camera_position: DVec3,
//     config: &OrbitCameraConfig,
// ) -> DQuat {
//     // Compute current latitude
//     let latitude = compute_latitude(camera_position);

//     // Compute blend factor (1.0 = full constraint, 0.0 = no constraint)
//     let constraint_factor = compute_roll_constraint_factor(
//         latitude,
//         config.roll_constraint_low_lat,
//         config.roll_constraint_high_lat,
//     );

//     if constraint_factor < 1e-6 {
//         // Near poles, no constraint needed
//         return rotation;
//     }

//     if constraint_factor > 1.0 - 1e-6 {
//         // Near equator, full constraint
//         return remove_roll_from_rotation(rotation, camera_position);
//     }

//     // Blend between constrained and unconstrained
//     let no_roll = remove_roll_from_rotation(rotation, camera_position);
//     DQuat::slerp(rotation, no_roll, constraint_factor)
// }

/// TODO: new plan
///
/// 0. compute the yaw angle of the camera in the NED frame
/// 1. read up on swing-twist decomposition and figure out if it has better precision than
///    DQuat::from_rotation_arc(). If it does then try it, otherwise keep using from_rotation_arc().
///    The point is that we need a rotation which moves the rig along a great circle towards the
///    target point.
/// 2. apply this rotation (scaled by timestep of course) to the position and the orbit rotation
/// 3. somehow modify the outer rotation such that it's north-up, while preserving the camera's new
///    yaw angle in the NED frame.
/// 4. interpolate between the camera's new yaw angle angle and the previous one based on latitude.
///
/// (3) is the hard part. After rotating the camera rig (aka orbit frame), we compute the frame's
/// roll angle I guess? and then apply the opposite of that rotation to the orbit frame, and THEN
/// apply that rotation, transformed into the inner frame, to the camera itself. The camera should
/// end up with the same exact global rotation, but the orbit frame will have zero roll
/// (i.e. north up).
///
///
///
///
/// Additional problems to fix:
///
/// - zooming doesn't hold the zoom point under the cursor fixed. It *eventually* reaches that, but
///   the angular rate seems to be out of sync with the radial rate. Should probably just decouple
///   this from the panning code entirely, not use a rotation ref, just apply the entire correction
///   each frame until the zoom is complete.
///
/// - during a pan or zoom, we need to map screen space positions beyond the limb of the earth to
///   points around the back side of the earth (maybe as shown in whiteboard diagram).
///
/// -

fn update_center_rotation_ref(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    input: &OrbitCameraInputEvent,
    cursor_world_space: &Option<Vec3>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    dt: f32,
    gizmos: &mut Gizmos,
) {
    let Some(pan_delta) = input.pan_delta else {
        state.pan = None;
        return;
    };

    if input.pan_started {
        // User has started a pan interaction during this frame.
        if let Some(cursor_world_space) = cursor_world_space {
            let start_world_space = cursor_world_space.as_dvec3();

            state.pan = Some(PanState {
                // cursor_position is always be populated if pan_started is true
                start_screen_space: input.cursor_position.unwrap(),
                offset_screen_space: Vec2::ZERO,
                start_world_space,
                start_radius: start_world_space.length(),
            });
        } else {
            state.pan = None;
        }
    }

    if let Some(pan_state) = state.pan.as_mut() {
        if !input.pan_started {
            // If we have an absolute cursor position (inside window), use it directly
            // and sync offset_screen_space so the fallback is seamless when cursor leaves.
            // Otherwise, accumulate the raw MouseMotion delta.
            if let Some(cursor_pos) = input.cursor_position {
                pan_state.offset_screen_space = cursor_pos - pan_state.start_screen_space;
            } else {
                pan_state.offset_screen_space += pan_delta;
            }
        }

        let current_screen_pos = pan_state.start_screen_space + pan_state.offset_screen_space;

        // TODO: try this: instead of interpolating in screen space, let's imagine a plane slicing
        // the earth. A plane can be fully constrained by two intersecting rays - where are they?
        // - camera to the original grab point where we started dragging
        // - camera to the current mouse position projected onto the surface of the earth
        //
        // This plane will not necessarily pass through the center of the earth (i.e. it won't
        // necessarily be normal to the earth's surface).
        //
        // Once we have this plane, we want to rotate the earth on an axis which is normal to it.
        // Of course, that axis also needs to pass through the center of the earth, so it's fully
        // defined.

        // Update: after more thought, I am convinced that the method above will behave better than
        // the original method of using DQuat::from_rotation_arc() and then DQuat::slerp() to move
        // the grab point towards the cursor. However, it will have the same problem of almost every
        // drag direction resulting in the yaw angle changing (i.e. North will not remain up).
        // However, I have a new idea for that as well: after applying this plane-normal rotation,
        // we just need to figure out what the yaw deviation is, and then rotate the camera center
        // about an axis from the center of the earth to the current position, by that same angle.
        // This should result in a motion where both of these constraints are maintained:
        // - The grab point moves along a straight line in screen space (since a straight line in
        //   world space will project to a straight line in screen space)
        // - The camera rig's yaw angle in the NED frame remains constant (i.e. North remains up)

        // Project start_world_space into screen space so we can interpolate toward the
        // cursor in 2D for smooth convergence.
        let interpolated_screen_pos = if let Ok(projected_start) =
            camera.world_to_viewport(camera_transform, pan_state.start_world_space.as_vec3())
        {
            let smoothing = (1.0 - (-config.pan_smoothing * dt as f64).exp()) as f32;
            projected_start + smoothing * (current_screen_pos - projected_start)
        } else {
            current_screen_pos
        };

        // --- Plane-normal rotation ---
        // Two rays from the camera (to grab point + to cursor point) define a plane.
        // We rotate the Earth about this plane's normal (through the origin).
        // This produces straight-line screen-space motion of the grab point.

        let camera_pos_world_space = camera_transform.translation().as_dvec3();

        // Get cursor-on-sphere, using nearest_point_on_sphere as fallback for misses
        let Some(cursor_view_ray) = camera
            .viewport_to_world(camera_transform, interpolated_screen_pos)
            .ok()
        else {
            return;
        };
        let cursor_on_sphere = nearest_point_on_sphere_f64(
            camera_pos_world_space,
            cursor_view_ray.direction.as_dvec3().normalize(),
            DVec3::ZERO,
            pan_state.start_radius,
        );

        let grab_point_view_dir = pan_state.start_world_space - camera_pos_world_space;
        let cursor_view_dir = cursor_on_sphere - camera_pos_world_space;

        // Plane normal from cross product of the two view rays
        let mut rotation_axis = grab_point_view_dir.cross(cursor_view_dir);
        let rotation_axis_length_squared = rotation_axis.length_squared();

        state.center_rotation_ref = if rotation_axis_length_squared < 1e-20 {
            DQuat::IDENTITY
        } else {
            rotation_axis /= rotation_axis_length_squared.sqrt();

            // Rotation angle about rotation_axis: project both sphere points onto the
            // plane perpendicular to rotation_axis, then measure the signed angle
            // between projections.
            // let start_projected = pan_state.start_world_space
            //     - rotation_axis * pan_state.start_world_space.dot(rotation_axis);
            // let cursor_projected =
            //     cursor_on_sphere - rotation_axis * cursor_on_sphere.dot(rotation_axis);
            // let angle = f64::atan2(
            //     cursor_projected.cross(start_projected).dot(rotation_axis),
            //     cursor_projected.dot(start_projected),
            // );
            let angle = pan_state.start_world_space.angle_between(cursor_on_sphere);

            let plane_rotation = DQuat::from_axis_angle(rotation_axis, angle);

            // --- Yaw correction (north-up preservation) ---
            // Measure how much "north" rotated due to the plane-normal rotation,
            // then compensate with a rotation about the radial axis.

            let old_pos = state.center_rotation * DVec3::new(0.0, 0.0, config.earth_radius as f64);
            let radial_old = old_pos.normalize();
            let north_old = DVec3::Y - radial_old * DVec3::Y.dot(radial_old); // north at old position

            let new_pos = plane_rotation * old_pos;
            let radial_new = new_pos.normalize();
            let north_new = DVec3::Y - radial_new * DVec3::Y.dot(radial_new); // true north at new position
            let north_transported = plane_rotation * north_old; // where old-north ended up

            // Project transported-north into tangent plane at new position
            let north_transported_projected =
                north_transported - radial_new * north_transported.dot(radial_new);

            let north_new_len = north_new.length();
            let north_transported_len = north_transported_projected.length();

            let yaw_correction = if north_new_len < 1e-10 || north_transported_len < 1e-10 {
                // At or very near the poles, north is undefined
                DQuat::IDENTITY
            } else {
                // Signed angle between transported and true north, about the radial axis
                let yaw_deviation = f64::atan2(
                    north_transported_projected.cross(north_new).dot(radial_new),
                    north_transported_projected.dot(north_new),
                );

                // Latitude-based blend: relax near poles where north is ambiguous
                let latitude = compute_latitude(new_pos);
                let lat_deg = latitude.to_degrees().abs();
                let constraint = if lat_deg <= config.roll_constraint_low_lat {
                    1.0
                } else if lat_deg >= config.roll_constraint_high_lat {
                    0.0
                } else {
                    let blend_t = (lat_deg - config.roll_constraint_low_lat)
                        / (config.roll_constraint_high_lat - config.roll_constraint_low_lat);
                    1.0 - (3.0 * blend_t * blend_t - 2.0 * blend_t * blend_t * blend_t)
                };

                DQuat::from_axis_angle(radial_new, yaw_deviation * constraint)
            };
            yaw_correction * plane_rotation
        };

        // Debug gizmo for current cursor position on sphere
        gizmos.sphere(
            Isometry3d::from_translation(cursor_on_sphere.as_vec3()),
            0.05,
            Color::srgb(0.0, 1.0, 0.0),
        );

        // Debug gizmo for initial grab point on sphere
        gizmos.sphere(
            Isometry3d::from_translation(pan_state.start_world_space.as_vec3()),
            0.05,
            Color::srgb(1.0, 0.0, 0.0),
        );
    }
}

fn update_camera_center_transform(
    config: &OrbitCameraConfig,
    state: &mut OrbitCameraState,
    center_transform: &mut Transform,
) {
    let mut center_rotation_relative = DQuat::IDENTITY;
    if state.pan.is_some() {
        state.center_rotation = state.center_rotation_ref * state.center_rotation;
        center_rotation_relative = state.center_rotation_ref;
    }

    // Apply zoom rotation immediately (without smoothing) to maintain the constraint
    // that the world point stays under the cursor throughout the zoom
    if state.zoom.is_some() {
        state.center_rotation = state.zoom_rotation_reference * state.center_rotation;
        center_rotation_relative = state.zoom_rotation_reference * center_rotation_relative;
    }

    // Derive world-space center point from rotation
    state.center_position_world_space =
        state.center_rotation * DVec3::new(0.0, 0.0, config.earth_radius as f64);

    center_transform.translation = state.center_position_world_space.as_vec3();

    center_transform.rotate(-center_rotation_relative.as_quat());

    // center_transform.look_at(Vec3::ZERO, Vec3::Y);

    // TODO: blend this quaternion with the one we get by doing slerp()
    // Somehow we will need to eventually "transfer" the yaw rotation into the inner camera rotation
    // with the understanding that this rotation may not change in a continuous fashion.
    // Ideally, the camera rig's rotation will always have zero roll angle when possible, (i.e. not
    // near the poles) and the camera's local yaw angle will always match its global yaw angle,
    // we'll be able to interpret it as a compass bearing.
}

/// Position the camera in the camera rig's local space using orbit euler angles and radius.
fn update_camera_orbit_transform(state: &OrbitCameraState, camera_transform: &mut Transform) {
    let radius = state.radius.max(f64::EPSILON) as f32;
    let pitch = state.current_euler_angles.x.to_radians();
    let yaw = state.current_euler_angles.y.to_radians();

    let orbit_rotation = Quat::from_euler(EulerRot::ZXY, yaw, pitch, 0.0);
    camera_transform.translation = orbit_rotation * Vec3::new(0.0, 0.0, radius);
    camera_transform.rotation = orbit_rotation;
}

pub fn step(
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut input_reader: MessageReader<OrbitCameraInputEvent>,
    mut camera_containers: Query<
        (
            Entity,
            &OrbitCameraConfig,
            &mut OrbitCameraState,
            &OrbitCameraChildRef,
        ),
        With<OrbitCameraRig>,
    >,
    cameras: Query<(Entity, &Camera, &GlobalTransform)>,
    mut transforms: Query<&mut Transform>,
) {
    let Some(input) = input_reader.read().next() else {
        return;
    };

    let frame_dt = time.delta_secs().min(0.1);

    for (camera_container, config, mut state, child_ref) in &mut camera_containers {
        // Draw a wireframe sphere to help visualize camera movements
        gizmos.sphere(Vec3::ZERO, config.earth_radius as f32, Color::WHITE);

        // Get camera directly using the stored reference
        if let Ok((camera_entity, camera, camera_global_transform)) =
            cameras.get(child_ref.camera_entity)
        {
            // Calculate cursor world position if we have a pan start position

            // TODO: figure out which object has been grabbed. Is it the surface of the earth? 2D or
            // 3D terrain? Or a waypoint or something?
            // For now, we should assume it's the surface of a smooth spherical earth.

            // Whatever we intersect, we need to get a point in spherical coordinates (lat/lon/alt?)
            // which becomes a "handle" with which to rotate the ellipsoid. On subsequent frames, we
            // must then compute the lat/lon deltas needed to move that handle point onto the new
            // screen ray passing through the mouse position.
            let cursor_position_world_space = if let Some(cursor_position) = input.cursor_position {
                cursor_to_world_on_sphere_f64(
                    cursor_position,
                    camera,
                    camera_global_transform,
                    config.earth_radius,
                )
                .map(|v| v.as_vec3())
            } else {
                None
            };

            update_center_rotation_ref(
                config,
                &mut state,
                &input,
                &cursor_position_world_space,
                camera,
                camera_global_transform,
                frame_dt,
                &mut gizmos,
            );
            update_zoom(
                config,
                &mut state,
                &input,
                &cursor_position_world_space,
                camera,
                camera_global_transform,
                frame_dt,
                &mut gizmos,
            );
            update_orbit(config, &mut state, input.orbit_delta, frame_dt);

            // TODO: figure out if there's a cleaner way to get to these transforms, ew
            // let mut camera_center_transform = transforms.get_mut(camera_container).unwrap();
            update_camera_center_transform(
                config,
                &mut state,
                transforms.get_mut(camera_container).unwrap().as_mut(),
            );
            update_camera_orbit_transform(
                &state,
                transforms.get_mut(camera_entity).unwrap().as_mut(),
            );
        }
    }
}
