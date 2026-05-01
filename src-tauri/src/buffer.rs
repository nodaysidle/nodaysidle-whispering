use std::collections::VecDeque;

pub struct RollingBuffer {
    samples: VecDeque<f32>,
    capacity: usize,
    start_sample: u64,
    next_sample: u64,
}

impl RollingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity),
            capacity,
            start_sample: 0,
            next_sample: 0,
        }
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.start_sample = self.next_sample;
    }

    pub fn push(&mut self, incoming: &[f32]) {
        for sample in incoming {
            if self.samples.len() == self.capacity {
                self.samples.pop_front();
                self.start_sample += 1;
            }
            self.samples.push_back(*sample);
            self.next_sample += 1;
        }
    }

    pub fn start_sample(&self) -> u64 {
        self.start_sample
    }

    pub fn next_sample(&self) -> u64 {
        self.next_sample
    }

    pub fn slice(&self, start: u64, end: u64) -> Vec<f32> {
        let start = start.max(self.start_sample).min(self.next_sample);
        let end = end.max(start).min(self.next_sample);
        let offset = (start - self.start_sample) as usize;
        let len = (end - start) as usize;
        self.samples
            .iter()
            .skip(offset)
            .take(len)
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::RollingBuffer;

    #[test]
    fn keeps_only_capacity_and_tracks_absolute_samples() {
        let mut buffer = RollingBuffer::new(4);
        buffer.push(&[1.0, 2.0, 3.0]);
        buffer.push(&[4.0, 5.0, 6.0]);

        assert_eq!(buffer.start_sample(), 2);
        assert_eq!(buffer.next_sample(), 6);
        assert_eq!(buffer.slice(0, 6), vec![3.0, 4.0, 5.0, 6.0]);
        assert_eq!(buffer.slice(3, 5), vec![4.0, 5.0]);
    }
}
