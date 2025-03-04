// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    math::{prelude::*, DVec2, DVec3},
    prelude::{Camera3dBundle, ReflectDefault},
    reflect::Reflect,
    render::camera::Camera,
    time::Time,
    transform::components::Transform,
};

use std::cmp;

use crate::orbit_camera::{events::*, state::*};

use super::config::OrbitCameraConfig;

fn distance_to_zoom_level(distance: f64) -> f64 {
    -distance.ln()
}

fn zoom_level_to_distance(zoom_level: f64) -> f64 {
    (-zoom_level).exp()
}

fn get_lat_lon_offset_from_world_space_offset(pan_offset_world_space: &DVec3) -> DVec3 {
    // See AbstractMap.WorldToGeoPosition()
    // For quadtree implementation of the map, the map scale needs to be compensated for.

    // var scaleFactor = Mathf.Pow(2f, (MapboxMap.InitialZoom - MapboxMap.AbsoluteZoom));
    // Vector3 offsetLocalMapboxObject = MapboxMap.Root.InverseTransformPoint(offsetWorldSpace) / (MapboxMap.WorldRelativeScale * scaleFactor);
    // return Mapbox.Unity.Utilities.Conversions.MetersToLatLon(
    //     new Mapbox.Utils.Vector2d(offsetLocalMapboxObject.x, offsetLocalMapboxObject.z)
    // );

    DVec3::new(0.0, 0.0, 0.0)
}

/**
 * Update position of camera focus point.
 *
 * Return true if the position changed.
 */
fn update_position(
    time: Res<Time>,
    mut state: OrbitCameraState,
    mut camera: &Camera,
    config: &OrbitCameraConfig,
) -> bool {
    // TODO: refactor this to use MapAdapter.SetMapCenterGeodetic()
    // if (FocusObject != null && LockPositionToObject) {
    //     positionError = transform.position != FocusObject.position;
    //     transform.position = FocusObject.position;
    //     return positionError;
    // } else if (MapboxMap != null) {
    // If we get close enough, stop updating.
    let offset_error = state.pan_offset_world_space - state.pan_offset_target;

    if offset_error.abs().element_sum() < 0.0001 {
        return false;
    }

    // state.pan_offset_world_space = Vector3.Lerp(state.pan_offset_world_space, _panOffsetTarget, pan_smoothing * Mathf.Min(Time.deltaTime, 0.02f));
    state.pan_offset_world_space = state.pan_offset_world_space.lerp(
        state.pan_offset_target,
        config.pan_smoothing * time.delta_secs_f64().min(0.02),
    );
    let new_lat_lon = state.drag_start_lat_lon
        + get_lat_lon_offset_from_world_space_offset(&state.pan_offset_world_space);

    // MapboxMap.SetCenterLatitudeLongitude(newLatLon);
    // } else {
    //     Vector3 positionError = transform.position - _focusPositionTarget;
    //     if (Mathf.Abs(positionError.x) + Mathf.Abs(positionError.y) + Mathf.Abs(positionError.z) < 0.00001f) { return false; }

    //     transform.position = Vector3.Lerp(transform.position, _focusPositionTarget, pan_smoothing * Mathf.Min(Time.deltaTime, 0.02f));
    // }
    return true;
}

// public void LateUpdate()
// {
//     // _mouseEnabled = !EventSystem.current.IsPointerOverGameObject(-1);

//     if (!LockPositionToObject)
//     {
//         UpdatePositionTarget();
//     }
//     UpdateZoomTarget();
//     UpdateRotationTarget();
//     if (LookTowardsObject)
//     {
//         LookTowardsTarget();
//     }
//     CheckRightClickThreshhold();

//     RotateTowardsTarget();

//     bool positionChanged = UpdatePosition();
//     bool zoomChanged =  UpdateZoom();
//     if (MapboxMap != null && (positionChanged || zoomChanged))
//     {
//         MapboxMap.UpdateMap();
//         MapAdapter.UpdateMapScale();
//     }
// }

pub fn control_system(
    time: Res<Time>,
    mut events: EventReader<OrbitCameraInput>,
    mut cameras: Query<(&OrbitCameraState, &Transform)>,
    config: Query<&OrbitCameraConfig>,
    // mut cameras: Query<(&OrbitCameraController, &mut LookTransform, &Transform)>,
) {

    // update_position(time, state, camera, &config);
    // Can only control one camera at a time.
    // let (mut transform, scene_transform) =
    //     if let Some((_, transform, scene_transform)) = cameras.iter_mut().find(|c| c.0.enabled) {
    //         (transform, scene_transform)
    //     } else {
    //         return;
    //     };

    // let mut look_angles = LookAngles::from_vector(-transform.look_direction().unwrap());
    // let mut radius_scalar = 1.0;
    // let radius = transform.radius();

    // let dt = time.delta_seconds();
    // for event in events.read() {
    //     match event {
    //         OrbitCameraInput::Orbit(delta) => {
    //             look_angles.add_yaw(dt * -delta.x);
    //             look_angles.add_pitch(dt * delta.y);
    //         }
    //         OrbitCameraInput::TranslateTarget(delta) => {
    //             let right_dir = scene_transform.rotation * -Vec3::X;
    //             let up_dir = scene_transform.rotation * Vec3::Y;
    //             transform.target += dt * delta.x * right_dir + dt * delta.y * up_dir;
    //         }
    //         OrbitCameraInput::Zoom(scalar) => {
    //             radius_scalar *= scalar;
    //         }
    //     }
    // }

    // look_angles.assert_not_looking_up();

    // let new_radius = (radius_scalar * radius).min(1000000.0).max(0.001);
    // transform.eye = transform.target + new_radius * look_angles.unit_vector();
}
