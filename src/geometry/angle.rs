/// Provides calculation of angles.
#[derive(Debug, Clone, Copy)]
pub struct Angle(f64);

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Angle {}

impl Angle {
    /// Create an angle from the radian.
    pub fn new(radian: f64) -> Self {
        Self(radian).normalize()
    }

    /// Get the radian.
    pub fn radian(&self) -> f64 {
        self.0
    }

    /// Normalize to the range of (-PI, PI].
    fn normalize(&self) -> Self {
        let radian = self.0.rem_euclid(2.0 * std::f64::consts::PI);
        let radian = if radian > std::f64::consts::PI {
            radian - 2.0 * std::f64::consts::PI
        } else {
            radian
        };
        Self(radian)
    }

    /// Calculate the clockwise angle difference.
    fn diff_clockwise_to(&self, to: Self) -> f64 {
        let diff = to.0 - self.0;
        if diff < 0.0 {
            diff + 2.0 * std::f64::consts::PI
        } else {
            diff
        }
    }

    /// Calculate the counterclockwise angle difference.
    fn diff_counterclockwise_to(&self, to: Self) -> f64 {
        let diff = self.0 - to.0;
        if diff < 0.0 {
            diff + 2.0 * std::f64::consts::PI
        } else {
            diff
        }
    }

    /// Create an iterator of angles between two angles.
    fn iter_range_closer(&self, other: Self, step_num: usize) -> AngleIter {
        //let (rad0, rad1) = (Self::normalized(rad0), Self::normalized(rad1));
        let (rad_from, rad_to) = {
            let diff_clockwise = self.diff_clockwise_to(other);
            let diff_counterclockwise = self.diff_counterclockwise_to(other);
            if diff_clockwise < diff_counterclockwise {
                (self.0, self.0 + diff_clockwise)
            } else {
                (other.0, other.0 + diff_counterclockwise)
            }
        };

        AngleIter {
            rad_from,
            rad_to,
            step_num,
            step_current: 0,
        }
    }

    /// Create an iterator of angles around the specified angle.
    fn iter_range_around(&self, radian_range: f64, step_num: usize) -> AngleIter {
        let frac_radian_range_2 = radian_range / 2.0;
        Self::new(self.0 - frac_radian_range_2)
            .iter_range_closer(Self::new(self.0 + frac_radian_range_2), step_num)
    }
}

/// An iterator of angles.
struct AngleIter {
    rad_from: f64,
    rad_to: f64,
    step_num: usize,
    step_current: usize,
}

impl Iterator for AngleIter {
    type Item = Angle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step_current < self.step_num {
            let angle = self.rad_from
                + (self.rad_to - self.rad_from) * (self.step_current as f64)
                    / ((self.step_num - 1) as f64);
            self.step_current += 1;
            Some(Angle::new(angle))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_normalize() {
        assert_eq!(Angle::new(0.0).0, 0.0);
        assert_eq!(Angle::new(std::f64::consts::PI).0, std::f64::consts::PI);
        assert_eq!(Angle::new(2.0 * std::f64::consts::PI).0, 0.0);
        assert_eq!(Angle::new(-std::f64::consts::PI).0, std::f64::consts::PI);
        assert_eq!(Angle::new(-2.0 * std::f64::consts::PI).0, 0.0);
    }

    #[test]
    fn test_angle_diff_clockwise_to() {
        assert_eq!(
            Angle::new(0.0).diff_clockwise_to(Angle::new(std::f64::consts::PI)),
            std::f64::consts::PI
        );
        assert_eq!(
            Angle::new(std::f64::consts::PI).diff_clockwise_to(Angle::new(0.0)),
            std::f64::consts::PI
        );
        assert_eq!(
            Angle::new(0.0).diff_clockwise_to(Angle::new(0.5 * std::f64::consts::PI)),
            0.5 * std::f64::consts::PI
        );
        assert_eq!(
            Angle::new(0.5 * std::f64::consts::PI).diff_clockwise_to(Angle::new(0.0)),
            1.5 * std::f64::consts::PI
        );
    }

    #[test]
    fn test_angle_diff_counterclockwise() {
        assert_eq!(
            Angle::new(0.0).diff_counterclockwise_to(Angle::new(std::f64::consts::PI)),
            std::f64::consts::PI
        );
        assert_eq!(
            Angle::new(std::f64::consts::PI).diff_counterclockwise_to(Angle::new(0.0)),
            std::f64::consts::PI
        );
        assert_eq!(
            Angle::new(0.0).diff_counterclockwise_to(Angle::new(0.5 * std::f64::consts::PI)),
            1.5 * std::f64::consts::PI
        );
        assert_eq!(
            Angle::new(0.5 * std::f64::consts::PI).diff_counterclockwise_to(Angle::new(0.0)),
            0.5 * std::f64::consts::PI
        );
    }

    #[test]
    fn test_angle_iter_range_closer() {
        let mut iter = Angle::new(0.0).iter_range_closer(Angle::new(std::f64::consts::PI * 0.5), 5);
        assert_eq!(iter.next(), Some(Angle::new(0.0)));
        assert_eq!(iter.next(), Some(Angle::new(0.125 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.25 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.375 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.5 * std::f64::consts::PI)));
        assert_eq!(iter.next(), None);

        let mut iter = Angle::new(std::f64::consts::PI * 0.5).iter_range_closer(Angle::new(0.0), 5);
        assert_eq!(iter.next(), Some(Angle::new(0.0)));
        assert_eq!(iter.next(), Some(Angle::new(0.125 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.25 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.375 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.5 * std::f64::consts::PI)));
        assert_eq!(iter.next(), None);

        let mut iter = Angle::new(std::f64::consts::PI * 1.8)
            .iter_range_closer(Angle::new(std::f64::consts::PI * 1.4), 5);
        assert_eq!(iter.next(), Some(Angle::new(1.4 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.5 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.6 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.7 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.8 * std::f64::consts::PI)));
        assert_eq!(iter.next(), None);

        let mut iter = Angle::new(0.0).iter_range_closer(Angle::new(std::f64::consts::PI * 1.5), 5);
        assert_eq!(iter.next(), Some(Angle::new(1.5 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.625 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.75 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.875 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(2.0 * std::f64::consts::PI)));
        assert_eq!(iter.next(), None);
    }
}
