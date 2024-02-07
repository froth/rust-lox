use std::cmp::max;

use miette::SourceSpan;

pub trait SourceSpanExtensions {
    fn until(&self, right: Self) -> Self;
}

impl SourceSpanExtensions for SourceSpan {
    fn until(&self, right: Self) -> Self {
        assert!(self.offset() <= right.offset());
        let len = max(right.offset() + right.len() - 1, self.len());
        (self.offset(), len).into()
    }
}

#[cfg(test)]
mod source_span_extension_tests {
    use miette::SourceSpan;

    use super::SourceSpanExtensions;

    #[test]
    fn non_overlapping() {
        let left: SourceSpan = (1, 1).into();
        let right: SourceSpan = (3, 2).into();
        let combined = left.until(right);
        assert_eq!(combined.offset(), 1);
        assert_eq!(combined.len(), 4);
    }

    #[test]
    fn overlapping() {
        let left: SourceSpan = (1, 10).into();
        let right: SourceSpan = (8, 2).into();
        let combined = left.until(right);
        assert_eq!(combined.offset(), 1);
        assert_eq!(combined.len(), 10);
    }
}
