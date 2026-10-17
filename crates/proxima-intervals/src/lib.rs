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
pub struct Interval<T: PartialOrd + Clone> {
    left: Bound<T>,
    right: Bound<T>,
}

impl<T: PartialOrd + Clone> Interval<T> {
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

    fn select_bound(a: &Bound<T>, b: &Bound<T>, take_larger: bool) -> Option<Bound<T>> {
        use Bound::*;

        match (a, b) {
            (Unbounded, Unbounded) => Some(Unbounded),
            (Unbounded, _) => Some(b.clone()),
            (_, Unbounded) => Some(a.clone()),
            (Included(x) | Excluded(x), Included(y) | Excluded(y)) => match x.partial_cmp(y)? {
                Ordering::Less => Some(if take_larger { b.clone() } else { a.clone() }),
                Ordering::Greater => Some(if take_larger { a.clone() } else { b.clone() }),
                Ordering::Equal => Some(if matches!(a, Excluded(_)) {
                    a.clone()
                } else if matches!(b, Excluded(_)) {
                    b.clone()
                } else {
                    a.clone()
                }),
            },
        }
    }

    pub fn intersection(&self, other: &Interval<T>) -> Option<Self> {
        let left = Self::select_bound(&self.left, &other.left, true)?;
        let right = Self::select_bound(&self.right, &other.right, false)?;

        let is_valid = match (&left, &right) {
            (Bound::Unbounded, _) | (_, Bound::Unbounded) => true,
            (Bound::Included(l) | Bound::Excluded(l), Bound::Included(r) | Bound::Excluded(r)) => {
                match l.partial_cmp(r)? {
                    Ordering::Less => true,
                    Ordering::Equal => {
                        matches!((&left, &right), (Bound::Included(_), Bound::Included(_)))
                    }
                    Ordering::Greater => false,
                }
            }
        };

        is_valid.then_some(Self { left, right })
    }
}

impl Interval<f64> {
    pub fn len(&self) -> Option<f64> {
        match (&self.left, &self.right) {
            (
                Bound::Included(left) | Bound::Excluded(left),
                Bound::Included(right) | Bound::Excluded(right),
            ) => Some(right - left),
            _ => None,
        }
    }
}

impl<T: PartialOrd + Display + Clone> Display for Interval<T> {
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
pub struct IntervalSet<T: PartialOrd + Clone> {
    intervals: Vec<Interval<T>>,
}

impl<T: PartialOrd + Clone> IntervalSet<T> {
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

impl<T: PartialOrd + Clone> Default for IntervalSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialOrd + Clone + Display> Display for IntervalSet<T> {
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
    fn constructors_set_expected_bounds() {
        assert_eq!(Interval::<i32>::all().left(), &Bound::Unbounded);
        assert_eq!(Interval::<i32>::all().right(), &Bound::Unbounded);
        assert_eq!(Interval::closed(1, 4).unwrap().to_string(), "[1, 4]");
        assert_eq!(Interval::open(1, 4).unwrap().to_string(), "(1, 4)");
        assert_eq!(Interval::left_closed(1).to_string(), "[1, +inf)");
        assert_eq!(Interval::right_closed(4).to_string(), "(-inf, 4]");
        assert_eq!(Interval::left_open(1).to_string(), "(1, +inf)");
        assert_eq!(Interval::right_open(4).to_string(), "(-inf, 4)");
        assert_eq!(
            Interval::left_open_right_closed(1, 4).unwrap().to_string(),
            "(1, 4]"
        );
        assert_eq!(
            Interval::left_closed_right_open(1, 4).unwrap().to_string(),
            "[1, 4)"
        );
    }

    #[test]
    fn cut_returns_the_overlapping_interval() {
        let left = Interval::closed(0, 5).unwrap();
        let right = Interval::closed(3, 8).unwrap();

        assert_eq!(left.intersection(&right).unwrap().to_string(), "[3, 5]");
        assert_eq!(right.intersection(&left).unwrap().to_string(), "[3, 5]");
    }

    #[test]
    fn cut_handles_unbounded_right_endpoints() {
        let unbounded_right = Interval::left_closed(0);
        let finite = Interval::closed(10, 20).unwrap();

        assert_eq!(
            unbounded_right.intersection(&finite).unwrap().to_string(),
            "[10, 20]"
        );
        assert_eq!(
            finite.intersection(&unbounded_right).unwrap().to_string(),
            "[10, 20]"
        );
    }

    #[test]
    fn cut_handles_unbounded_left_endpoints() {
        let unbounded_left = Interval::right_closed(10);
        let finite = Interval::closed(0, 20).unwrap();

        assert_eq!(unbounded_left.intersection(&finite).unwrap().to_string(), "[0, 10]");
        assert_eq!(finite.intersection(&unbounded_left).unwrap().to_string(), "[0, 10]");
    }

    #[test]
    fn cut_preserves_open_boundaries_and_singletons() {
        let right_open_at_one = Interval::left_closed_right_open(0, 1).unwrap();
        let closed_at_one = Interval::closed(1, 2).unwrap();
        let open_at_zero = Interval::left_open_right_closed(0, 3).unwrap();
        let closed_from_zero = Interval::closed(0, 4).unwrap();

        assert!(right_open_at_one.intersection(&closed_at_one).is_none());
        assert_eq!(
            open_at_zero.intersection(&closed_from_zero).unwrap().to_string(),
            "(0, 3]"
        );
        assert_eq!(
            Interval::closed(0, 1)
                .unwrap()
                .intersection(&closed_at_one)
                .unwrap()
                .to_string(),
            "[1, 1]"
        );
    }

    #[test]
    fn cut_returns_none_for_disjoint_intervals() {
        let left = Interval::closed(0, 1).unwrap();
        let right = Interval::closed(2, 3).unwrap();

        assert!(left.intersection(&right).is_none());
        assert!(right.intersection(&left).is_none());
    }

    #[test]
    fn float_len_is_defined_only_for_finite_intervals() {
        assert_eq!(Interval::closed(1.5, 4.0).unwrap().len(), Some(2.5));
        assert_eq!(Interval::closed(1.0, 1.0).unwrap().len(), Some(0.0));
        assert_eq!(Interval::left_closed(1.0).len(), None);
        assert_eq!(Interval::right_closed(1.0).len(), None);
        assert_eq!(Interval::<f64>::all().len(), None);
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
