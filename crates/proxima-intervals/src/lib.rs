use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntervalError {
    BoundsNotIncreasing,
    EmptyInterval,
    IncomparableBounds,
}

/// A finite endpoint or an unbounded endpoint of an interval.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Bound<T: PartialOrd> {
    Unbounded,
    Included(T),
    Excluded(T),
}

/// An interval with independent left and right endpoint semantics.
///
/// `Unbounded` means negative infinity on the left and positive infinity on
/// the right. When both endpoints are finite, the left value must be less than
/// the right value, except that equal included endpoints form a singleton.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Interval<T: PartialOrd> {
    left: Bound<T>,
    right: Bound<T>,
}

impl<T: PartialOrd> Interval<T> {
    pub fn new(left: Bound<T>, right: Bound<T>) -> Result<Self, IntervalError> {
        let finite_bounds = match (&left, &right) {
            (
                Bound::Included(left) | Bound::Excluded(left),
                Bound::Included(right) | Bound::Excluded(right),
            ) => Some((left, right)),
            _ => None,
        };

        if let Some((left_value, right_value)) = finite_bounds {
            match left_value.partial_cmp(right_value) {
                Some(Ordering::Less) => {}
                Some(Ordering::Equal)
                    if matches!((&left, &right), (Bound::Included(_), Bound::Included(_))) => {}
                Some(Ordering::Equal) => return Err(IntervalError::EmptyInterval),
                Some(Ordering::Greater) => {
                    return Err(IntervalError::BoundsNotIncreasing);
                }
                None => return Err(IntervalError::IncomparableBounds),
            }
        }

        Ok(Self { left, right })
    }

    pub fn all() -> Self {
        Self {
            left: Bound::Unbounded,
            right: Bound::Unbounded,
        }
    }

    pub fn left(&self) -> &Bound<T> {
        &self.left
    }

    pub fn right(&self) -> &Bound<T> {
        &self.right
    }

    pub fn closed(left: T, right: T) -> Result<Self, IntervalError> {
        Self::new(Bound::Included(left), Bound::Included(right))
    }

    pub fn open(left: T, right: T) -> Result<Self, IntervalError> {
        Self::new(Bound::Excluded(left), Bound::Excluded(right))
    }

    pub fn left_closed(left: T) -> Self {
        Self {
            left: Bound::Included(left),
            right: Bound::Unbounded,
        }
    }

    pub fn right_closed(right: T) -> Self {
        Self {
            left: Bound::Unbounded,
            right: Bound::Included(right),
        }
    }

    pub fn left_open(left: T) -> Self {
        Self {
            left: Bound::Excluded(left),
            right: Bound::Unbounded,
        }
    }

    pub fn right_open(right: T) -> Self {
        Self {
            left: Bound::Unbounded,
            right: Bound::Excluded(right),
        }
    }

    pub fn left_open_right_closed(left: T, right: T) -> Result<Self, IntervalError> {
        Self::new(Bound::Excluded(left), Bound::Included(right))
    }

    pub fn left_closed_right_open(left: T, right: T) -> Result<Self, IntervalError> {
        Self::new(Bound::Included(left), Bound::Excluded(right))
    }
}

impl Interval<f64> {
    pub fn len(&self) -> Option<f64> {
        match (&self.left, &self.right) {
            (Bound::Included(left) | Bound::Excluded(left), Bound::Included(right) | Bound::Excluded(right)) => Some(right - left),
            _ => None,
        }
    }
}

impl<T: PartialOrd + Display> Display for Interval<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match &self.left {
            Bound::Unbounded => formatter.write_str("(-inf")?,
            Bound::Included(value) => write!(formatter, "[{value}")?,
            Bound::Excluded(value) => write!(formatter, "({value}")?,
        }

        formatter.write_str(", ")?;

        match &self.right {
            Bound::Unbounded => formatter.write_str("+inf)")?,
            Bound::Included(value) => write!(formatter, "{value}]")?,
            Bound::Excluded(value) => write!(formatter, "{value})")?,
        }

        Ok(())
    }
}

/// A collection of pairwise-disjoint intervals, in ascending order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntervalSet<T: PartialOrd> {
    intervals: Vec<Interval<T>>,
}

impl<T: PartialOrd> IntervalSet<T> {
    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }

    /// Stores intervals that the caller has already sorted and made disjoint.
    /// This initial generic API does not compare or normalize their endpoints.
    pub fn from_disjoint_intervals(intervals: Vec<Interval<T>>) -> Self {
        Self { intervals }
    }

    pub fn intervals(&self) -> &[Interval<T>] {
        &self.intervals
    }

    pub fn into_intervals(self) -> Vec<Interval<T>> {
        self.intervals
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn len(&self) -> usize {
        self.intervals.len()
    }
}

impl<T: PartialOrd> Default for IntervalSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialOrd + Display> Display for IntervalSet<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("{")?;

        for (index, interval) in self.intervals.iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "{interval}")?;
        }

        formatter.write_str("}")
    }
}

#[cfg(test)]
mod tests {
    use super::{Bound, Interval, IntervalSet};

    #[test]
    fn formats_closed_and_open_endpoints() {
        let interval = Interval::new(Bound::Included(-1), Bound::Excluded(3)).unwrap();

        assert_eq!(interval.to_string(), "[-1, 3)");
    }

    #[test]
    fn formats_unbounded_endpoints_as_infinities() {
        assert_eq!(Interval::<i32>::all().to_string(), "(-inf, +inf)");
        assert_eq!(
            Interval::new(Bound::Unbounded, Bound::Included(5))
                .unwrap()
                .to_string(),
            "(-inf, 5]"
        );
    }

    #[test]
    fn formats_interval_sets_and_empty_sets() {
        let intervals = vec![
            Interval::new(Bound::Included(-1), Bound::Included(0)).unwrap(),
            Interval::new(Bound::Excluded(2), Bound::Unbounded).unwrap(),
        ];
        let set = IntervalSet::from_disjoint_intervals(intervals);

        assert_eq!(set.to_string(), "{[-1, 0], (2, +inf)}");
        assert_eq!(IntervalSet::<i32>::new().to_string(), "{}");
    }

    #[test]
    fn supports_non_numeric_endpoint_types() {
        let interval = Interval::new(
            Bound::Included(String::from("alpha")),
            Bound::Excluded(String::from("omega")),
        )
        .unwrap();

        assert_eq!(interval.to_string(), "[alpha, omega)");
    }

    #[test]
    fn allows_equal_included_bounds_as_singleton() {
        let interval = Interval::new(Bound::Included(0), Bound::Included(0)).unwrap();

        assert_eq!(interval.to_string(), "[0, 0]");
    }

    #[test]
    fn rejects_empty_or_reversed_bounds() {
        assert_eq!(
            Interval::new(Bound::Included(1), Bound::Excluded(1)),
            Err(super::IntervalError::EmptyInterval)
        );
        assert_eq!(
            Interval::new(Bound::Excluded(1), Bound::Included(1)),
            Err(super::IntervalError::EmptyInterval)
        );
        assert_eq!(
            Interval::new(Bound::Excluded(1), Bound::Excluded(1)),
            Err(super::IntervalError::EmptyInterval)
        );
        assert_eq!(
            Interval::new(Bound::Included(2), Bound::Included(1)),
            Err(super::IntervalError::BoundsNotIncreasing)
        );
    }

    #[test]
    fn rejects_incomparable_float_bounds() {
        assert_eq!(
            Interval::new(Bound::Included(f64::NAN), Bound::Included(1.0)),
            Err(super::IntervalError::IncomparableBounds)
        );
    }
}
