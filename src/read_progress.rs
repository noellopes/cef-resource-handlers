pub(crate) struct ReadProgress {
    pub(crate) offset: usize,
    pub(crate) length: Option<usize>,
}

impl ReadProgress {
    pub(crate) fn new(length: Option<usize>) -> Self {
        Self { offset: 0, length }
    }

    pub(crate) fn bytes_available(&self, needed: usize) -> usize {
        match self.length {
            Some(length) => length.saturating_sub(self.offset).min(needed),
            None => needed,
        }
    }

    pub(crate) fn advance(&mut self, bytes_to_advance: usize) -> usize {
        let bytes_available = self.bytes_available(bytes_to_advance);
        self.offset += bytes_available;
        bytes_available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_progress_new() {
        let progress = ReadProgress::new(Some(100));
        assert_eq!(progress.offset, 0);
        assert_eq!(progress.length, Some(100));

        let progress = ReadProgress::new(None);
        assert_eq!(progress.offset, 0);
        assert_eq!(progress.length, None);
    }

    #[test]
    fn test_bytes_available() {
        // Length unknown - should return needed
        let progress = ReadProgress {
            offset: 5,
            length: None,
        };

        assert_eq!(
            progress.bytes_available(10),
            10,
            "Should return needed bytes when length is unknown"
        );

        // Normal case - enough bytes available
        let progress = ReadProgress {
            offset: 5,
            length: Some(10),
        };
        assert_eq!(
            progress.bytes_available(4),
            4,
            "Should return requested bytes when available"
        );

        // Limited bytes available
        let progress = ReadProgress {
            offset: 8,
            length: Some(10),
        };
        assert_eq!(
            progress.bytes_available(5),
            2,
            "Should return only remaining bytes when limited"
        );

        // No bytes available - at end
        let progress = ReadProgress {
            offset: 10,
            length: Some(10),
        };
        assert_eq!(
            progress.bytes_available(5),
            0,
            "Should return 0 when at end of resource"
        );

        // No bytes available - position exceeds size
        let progress = ReadProgress {
            offset: 15,
            length: Some(10),
        };
        assert_eq!(
            progress.bytes_available(5),
            0,
            "Should return 0 when position exceeds size"
        );

        // Large values
        let progress = ReadProgress {
            offset: usize::MAX - 10,
            length: Some(usize::MAX),
        };
        assert_eq!(
            progress.bytes_available(20),
            10,
            "Should handle large values correctly"
        );
    }
}
