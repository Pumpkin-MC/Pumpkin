#[derive(Clone, Copy, Default, Debug)]
pub struct Sample {
    pub y: f64,
    pub y_rot: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct DragonFlightHistory {
    samples: [Sample; 64],
    head: i32,
}

impl Default for DragonFlightHistory {
    fn default() -> Self {
        Self {
            samples: [Sample { y: 0.0, y_rot: 0.0 }; 64],
            head: -1,
        }
    }
}

impl DragonFlightHistory {
    /// Java: `DragonFlightHistory.copyFrom()` — copies all samples and head
    /// position from another history, used for syncing client-side interpolation.
    pub const fn copy_from(&mut self, other: &Self) {
        self.samples = other.samples;
        self.head = other.head;
    }

    pub const fn record(&mut self, y: f64, y_rot: f32) {
        let frame = Sample { y, y_rot };
        if self.head < 0 {
            self.samples = [frame; 64];
        }

        self.head += 1;
        if self.head >= 64 {
            self.head = 0;
        }
        self.samples[self.head as usize] = frame;
    }

    #[must_use]
    pub fn get(&self, offset: i32) -> Sample {
        if self.head < 0 {
            return Sample::default();
        }
        let index = ((self.head - offset) & 63) as usize;
        self.samples[index]
    }
}
