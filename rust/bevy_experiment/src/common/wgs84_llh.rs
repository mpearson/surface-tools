use core::fmt;

/// A WGS84 geodetic position.
#[derive(Clone, Copy)]
pub struct Wgs84Llh {
    pub lat: f64,
    pub lon: f64,
    pub height: f64,
}

impl Wgs84Llh {
    /// All zeroes.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// Create a Wgs84Llh instance.
    #[inline(always)]
    #[must_use]
    pub const fn new(lat: f64, lon: f64, height: f64) -> Self {
        Self { lat, lon, height }
    }
}

impl Default for Wgs84Llh {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Wgs84Llh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(p) = f.precision() {
            write!(
                f,
                "[{:.*}, {:.*}, {:.*}]",
                p, self.lat, p, self.lon, p, self.height
            )
        } else {
            write!(f, "[{}, {}, {}]", self.lat, self.lon, self.height)
        }
    }
}

impl fmt::Debug for Wgs84Llh {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple(stringify!(Wgs84Llh))
            .field(&self.lat)
            .field(&self.lon)
            .field(&self.height)
            .finish()
    }
}
