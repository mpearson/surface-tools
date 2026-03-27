use bevy::math::{DQuat, DVec3};
use std::f64::consts::PI;

/// Compute the rotation that takes `from` to `to`, using unnormalized vectors
/// to preserve precision for near-coincident points at large distances from the origin.
pub fn great_circle_rotation(from: DVec3, to: DVec3) -> DQuat {
    let cross = from.cross(to);
    let cross_len = cross.length();
    let dot = from.dot(to);

    // Use cross product magnitude for the degenerate-case check instead of
    // dot / len_product, because the cross product retains the small-angle
    // information that the dot product loses at earth scale.
    // cross_len = |from||to|sin(θ), which is well-conditioned even for tiny θ
    // when |from| and |to| are large.
    let len_product = from.length() * to.length();

    if len_product < f64::EPSILON {
        return DQuat::IDENTITY;
    }

    if cross_len < len_product * f64::EPSILON {
        if dot >= 0.0 {
            // from ≈ to
            return DQuat::IDENTITY;
        } else {
            // from ≈ -to
            return DQuat::from_axis_angle(from.normalize().any_orthonormal_vector(), PI);
        }
    }

    let axis = cross / cross_len;
    // atan2 avoids the precision-losing division by len_product that asin would need.
    // atan2(|from||to|sin(θ), |from||to|cos(θ)) = θ, with the scale factors cancelling.
    let angle = f64::atan2(cross_len, dot);
    DQuat::from_axis_angle(axis, angle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::EulerRot;
    use rand::distr::StandardUniform;
    use rand::prelude::*;
    use std::f64::consts::TAU;

    const EARTH_RADIUS: f64 = 6378137.0;
    const TOLERANCE: f64 = 1e-6;

    #[test]
    fn precision_at_earth_scale() {
        let mut rng = rand::rng();

        let offset = 0.001;
        let mut max_error: f64 = 0.0;

        for _ in 0..5000 {
            let r0 = DQuat::from_euler(
                EulerRot::YXZ,
                rng.sample::<f64, _>(StandardUniform) * TAU,
                rng.sample::<f64, _>(StandardUniform) * TAU - PI,
                0.0,
            );

            let p1 = r0 * DVec3::new(0.0, 0.0, -EARTH_RADIUS);
            let p2 = r0 * DVec3::new(0.0, offset, -EARTH_RADIUS);

            let q = great_circle_rotation(p1, p2);
            let error = ((q * p1) - p2).length();
            max_error = max_error.max(error);
        }

        assert!(
            max_error < TOLERANCE,
            "worst error {max_error:.6e} exceeds {TOLERANCE:.6e}m threshold"
        );
    }
}
