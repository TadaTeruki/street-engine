use super::site::Site2D;

/// Represent an angle.
#[derive(Debug, Copy, Clone)]
pub struct Angle {
    radian: f64,
}

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        self.radian == other.radian
    }
}

impl Eq for Angle {}

impl Angle {
    /// Normalize to the range of (-PI, PI].
    /// The value which this stuructures holds must be normalized.
    fn normalize(&self) -> Self {
        let mut radian = self.radian.rem_euclid(2.0 * std::f64::consts::PI);
        if radian > std::f64::consts::PI {
            radian -= 2.0 * std::f64::consts::PI;
        }
        Self { radian }
    }

    /// Create an angle from a radian.
    pub fn new(radian: f64) -> Self {
        Self { radian }.normalize()
    }
    
    /// Create an angle from a site as a vector.
    pub fn from_site(site: &Site2D) -> Self {
        // atan2 is already normalized
        Self::new(site.y.atan2(site.x))
    }

    /// Calculate the angle between two sites.
    pub fn between_two_sites(from: &Site2D, to: &Site2D) -> Self {
        Self::from_site(&Site2D::new(to.x - from.x, to.y - from.y))
    }

    /// Calculate the clockwise angle difference to the other angle.
    fn diff_clockwise_to(&self, other: &Self) -> f64 {
        let diff = other.radian - self.radian;
        if diff < 0.0 {
            diff + 2.0 * std::f64::consts::PI
        } else {
            diff
        }
    }

    /// Calculate the counterclockwise angle difference to the other angle.
    fn diff_counterclockwise_to(&self, other: &Self) -> f64 {
        let diff = self.radian - other.radian;
        if diff < 0.0 {
            diff + 2.0 * std::f64::consts::PI
        } else {
            diff
        }
    }

    /// Create an iterator of angles from the current angle to the other angle with the number of steps.
    pub fn iter_range_closer(from: &Self, to: &Self, step_num: usize) -> AngleIter {
        let (from_radian, to_radian) = {
            let diff_clockwise = from.diff_clockwise_to(to);
            let diff_counterclockwise = from.diff_counterclockwise_to(to);
            if diff_clockwise < diff_counterclockwise {
                (from.radian, from.radian + diff_clockwise)
            } else {
                (to.radian, to.radian + diff_counterclockwise)
            }
        };

        AngleIter {
            from_radian,
            to_radian,
            step_num,
            step_current: 0,
        }
    }
}

pub struct AngleIter {
    from_radian: f64,
    to_radian: f64,
    step_num: usize,
    step_current: usize,
}

impl Iterator for AngleIter {
    type Item = Angle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step_current < self.step_num {
            let angle = Angle::new(
                self.from_radian
                    + (self.to_radian - self.from_radian) * (self.step_current as f64)
                        / ((self.step_num - 1) as f64),
            );
            self.step_current += 1;
            Some(angle)
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
        let angle = Angle::new(0.0);
        assert_eq!(angle.radian, 0.0);

        let angle = Angle::new(std::f64::consts::PI);
        assert_eq!(angle.radian, std::f64::consts::PI);

        let angle = Angle::new(2.0 * std::f64::consts::PI);
        assert_eq!(angle.radian, 0.0);

        let angle = Angle::new(-std::f64::consts::PI);
        assert_eq!(angle.radian, std::f64::consts::PI);

        let angle = Angle::new(-2.0 * std::f64::consts::PI);
        assert_eq!(angle.radian, 0.0);

        let angle = Angle::new(3.0 * std::f64::consts::PI);
        assert_eq!(angle.radian, std::f64::consts::PI);
    }

    #[test]
    fn test_angle_diff_clockwise_to() {
        let angle0 = Angle::new(0.0);
        let angle1 = Angle::new(std::f64::consts::PI);
        assert_eq!(angle0.diff_clockwise_to(&angle1), std::f64::consts::PI);

        let angle0 = Angle::new(std::f64::consts::PI);
        let angle1 = Angle::new(0.0);
        assert_eq!(angle0.diff_clockwise_to(&angle1), std::f64::consts::PI);

        let angle0 = Angle::new(0.0);
        let angle1 = Angle::new(0.5 * std::f64::consts::PI);
        assert_eq!(
            angle0.diff_clockwise_to(&angle1),
            0.5 * std::f64::consts::PI
        );

        let angle0 = Angle::new(0.5 * std::f64::consts::PI);
        let angle1 = Angle::new(0.0);
        assert_eq!(
            angle0.diff_clockwise_to(&angle1),
            1.5 * std::f64::consts::PI
        );
    }

    #[test]
    fn test_angle_diff_counterclockwise_to() {
        let angle0 = Angle::new(0.0);
        let angle1 = Angle::new(std::f64::consts::PI);
        assert_eq!(
            angle0.diff_counterclockwise_to(&angle1),
            std::f64::consts::PI
        );

        let angle0 = Angle::new(std::f64::consts::PI);
        let angle1 = Angle::new(0.0);
        assert_eq!(
            angle0.diff_counterclockwise_to(&angle1),
            std::f64::consts::PI
        );

        let angle0 = Angle::new(0.0);
        let angle1 = Angle::new(0.5 * std::f64::consts::PI);
        assert_eq!(
            angle0.diff_counterclockwise_to(&angle1),
            1.5 * std::f64::consts::PI
        );

        let angle0 = Angle::new(0.5 * std::f64::consts::PI);
        let angle1 = Angle::new(0.0);
        assert_eq!(
            angle0.diff_counterclockwise_to(&angle1),
            0.5 * std::f64::consts::PI
        );
    }

    #[test]
    fn test_angle_iter_range_closer() {
        let angle0 = Angle::new(0.0);
        let angle1 = Angle::new(std::f64::consts::PI * 0.5);
        let mut iter = Angle::iter_range_closer(&angle0, &angle1, 5);
        assert_eq!(iter.next(), Some(Angle::new(0.0)));
        assert_eq!(iter.next(), Some(Angle::new(0.125 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.25 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.375 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.5 * std::f64::consts::PI)));
        assert_eq!(iter.next(), None);

        let angle0 = Angle::new(std::f64::consts::PI * 0.5);
        let angle1 = Angle::new(0.0);
        let mut iter = Angle::iter_range_closer(&angle0, &angle1, 5);
        assert_eq!(iter.next(), Some(Angle::new(0.0)));
        assert_eq!(iter.next(), Some(Angle::new(0.125 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.25 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.375 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(0.5 * std::f64::consts::PI)));
        assert_eq!(iter.next(), None);

        let angle0 = Angle::new(std::f64::consts::PI * 1.8);
        let angle1 = Angle::new(std::f64::consts::PI * 1.4);
        let mut iter = Angle::iter_range_closer(&angle0, &angle1, 5);
        assert_eq!(iter.next(), Some(Angle::new(1.4 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.5 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.6 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.7 * std::f64::consts::PI)));
        assert_eq!(iter.next(), Some(Angle::new(1.8 * std::f64::consts::PI)));
        assert_eq!(iter.next(), None);
    }
}
