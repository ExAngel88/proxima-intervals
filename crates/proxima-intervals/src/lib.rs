use std::fmt::{self, Display, Formatter};

/// A finite endpoint or an unbounded endpoint of an interval.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Bound<T> {
    Unbounded,
    Included(T),
    Excluded(T),
}

/// An interval with independent left and right endpoint semantics.
///
/// `Unbounded` means negative infinity on the left and positive infinity on
/// the right. Endpoint ordering and validity are not checked by this type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Interval<T> {
    pub left: Bound<T>,
    pub right: Bound<T>,
}

impl<T> Interval<T> {
    pub fn new(left: Bound<T>, right: Bound<T>) -> Self {
        Self { left, right }
    }

    pub fn all() -> Self {
        Self::new(Bound::Unbounded, Bound::Unbounded)
    }
}

impl<T: Display> Display for Interval<T> {
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
pub struct IntervalSet<T> {
    intervals: Vec<Interval<T>>,
}

impl<T> IntervalSet<T> {
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

impl<T> Default for IntervalSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Display> Display for IntervalSet<T> {
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
        let interval = Interval::new(Bound::Included(-1), Bound::Excluded(3));

        assert_eq!(interval.to_string(), "[-1, 3)");
    }

    #[test]
    fn formats_unbounded_endpoints_as_infinities() {
        assert_eq!(Interval::<i32>::all().to_string(), "(-inf, +inf)");
        assert_eq!(
            Interval::new(Bound::Unbounded, Bound::Included(5)).to_string(),
            "(-inf, 5]"
        );
    }

    #[test]
    fn formats_interval_sets_and_empty_sets() {
        let intervals = vec![
            Interval::new(Bound::Included(-1), Bound::Included(0)),
            Interval::new(Bound::Excluded(2), Bound::Unbounded),
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
        );

        assert_eq!(interval.to_string(), "[alpha, omega)");
    }
}
