/// Provides calculation of angles.
struct Angle;

impl Angle {
    /// Normalize to the range of (-PI, PI].
    fn normalized(radian: f64) -> f64 {
        let radian = radian.rem_euclid(2.0 * std::f64::consts::PI);
        if radian > std::f64::consts::PI {
            radian - 2.0 * std::f64::consts::PI
        } else {
            radian
        }
    }

    /// Calculate the clockwise angle difference.
    fn diff_clockwise(rad_from: f64, rad_to: f64) -> f64 {
        //let diff = rad_to - rad_from;
        let diff = Self::normalized(rad_to) - Self::normalized(rad_from);
        if diff < 0.0 {
            diff + 2.0 * std::f64::consts::PI
        } else {
            diff
        }
    }

    /// Calculate the counterclockwise angle difference.
    fn diff_counterclockwise(rad_from: f64, rad_to: f64) -> f64 {
        let diff = Self::normalized(rad_from) - Self::normalized(rad_to);
        if diff < 0.0 {
            diff + 2.0 * std::f64::consts::PI
        } else {
            diff
        }
    }

    /// Create an iterator of angles between two angles.
    fn iter_range_closer(rad0: f64, rad1: f64, step_num: usize) -> AngleIter {
        let (rad0, rad1) = (Self::normalized(rad0), Self::normalized(rad1));
        let (rad_from, rad_to) = {
            let diff_clockwise = Self::diff_clockwise(rad0, rad1);
            let diff_counterclockwise = Self::diff_counterclockwise(rad0, rad1);
            if diff_clockwise < diff_counterclockwise {
                (rad0, rad0 + diff_clockwise)
            } else {
                (rad1, rad1 + diff_counterclockwise)
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
    fn iter_range_around(radian: f64, rad_range: f64, step_num: usize) -> AngleIter {
        Self::iter_range_closer(radian - rad_range / 2.0, radian + rad_range / 2.0, step_num)
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
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step_current < self.step_num {
            let angle = self.rad_from
                + (self.rad_to - self.rad_from) * (self.step_current as f64)
                    / ((self.step_num - 1) as f64);
            self.step_current += 1;
            Some(Angle::normalized(angle))
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
        assert_eq!(Angle::normalized(0.0), 0.0);
        assert_eq!(
            Angle::normalized(std::f64::consts::PI),
            std::f64::consts::PI
        );
        assert_eq!(Angle::normalized(2.0 * std::f64::consts::PI), 0.0);
        assert_eq!(
            Angle::normalized(-std::f64::consts::PI),
            std::f64::consts::PI
        );
        assert_eq!(Angle::normalized(-2.0 * std::f64::consts::PI), 0.0);
    }

    #[test]
    fn test_angle_diff_clockwise_to() {
        assert_eq!(
            Angle::diff_clockwise(0.0, std::f64::consts::PI),
            std::f64::consts::PI
        );
        assert_eq!(
            Angle::diff_clockwise(std::f64::consts::PI, 0.0),
            std::f64::consts::PI
        );
        assert_eq!(
            Angle::diff_clockwise(0.0, 0.5 * std::f64::consts::PI),
            0.5 * std::f64::consts::PI
        );
        assert_eq!(
            Angle::diff_clockwise(0.5 * std::f64::consts::PI, 0.0),
            1.5 * std::f64::consts::PI
        );
    }

    #[test]
    fn test_angle_diff_counterclockwise() {
        assert_eq!(
            Angle::diff_counterclockwise(0.0, std::f64::consts::PI),
            std::f64::consts::PI
        );
        assert_eq!(
            Angle::diff_counterclockwise(std::f64::consts::PI, 0.0),
            std::f64::consts::PI
        );
        assert_eq!(
            Angle::diff_counterclockwise(0.0, 0.5 * std::f64::consts::PI),
            1.5 * std::f64::consts::PI
        );
        assert_eq!(
            Angle::diff_counterclockwise(0.5 * std::f64::consts::PI, 0.0),
            0.5 * std::f64::consts::PI
        );
    }

    #[test]
    fn test_angle_iter_range_closer() {
        let mut iter = Angle::iter_range_closer(0.0, std::f64::consts::PI * 0.5, 5);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(0.125 * std::f64::consts::PI));
        assert_eq!(iter.next(), Some(0.25 * std::f64::consts::PI));
        assert_eq!(iter.next(), Some(0.375 * std::f64::consts::PI));
        assert_eq!(iter.next(), Some(0.5 * std::f64::consts::PI));
        assert_eq!(iter.next(), None);

        let mut iter = Angle::iter_range_closer(std::f64::consts::PI * 0.5, 0.0, 5);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(0.125 * std::f64::consts::PI));
        assert_eq!(iter.next(), Some(0.25 * std::f64::consts::PI));
        assert_eq!(iter.next(), Some(0.375 * std::f64::consts::PI));
        assert_eq!(iter.next(), Some(0.5 * std::f64::consts::PI));
        assert_eq!(iter.next(), None);

        let mut iter =
            Angle::iter_range_closer(std::f64::consts::PI * 1.8, std::f64::consts::PI * 1.4, 5);
        assert_eq!(
            iter.next(),
            Some(Angle::normalized(1.4 * std::f64::consts::PI))
        );
        assert_eq!(
            iter.next(),
            Some(Angle::normalized(1.5 * std::f64::consts::PI))
        );
        assert_eq!(
            iter.next(),
            Some(Angle::normalized(1.6 * std::f64::consts::PI))
        );
        assert_eq!(
            iter.next(),
            Some(Angle::normalized(1.7 * std::f64::consts::PI))
        );
        assert_eq!(
            iter.next(),
            Some(Angle::normalized(1.8 * std::f64::consts::PI))
        );
        assert_eq!(iter.next(), None);
    }
}
