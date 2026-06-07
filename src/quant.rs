pub struct SQ8Quantizer {
    pub min: f32,
    pub max: f32,
    pub scale: f32,
}

impl SQ8Quantizer {
    pub fn new(min: f32, max: f32) -> Self {
        let scale = if max > min { (max - min) / 255.0 } else { 1.0 };
        SQ8Quantizer { min, max, scale }
    }

    pub fn quantize(&self, data: &[f32]) -> Vec<u8> {
        data.iter()
            .map(|&x| {
                let clamped = x.clamp(self.min, self.max);
                ((clamped - self.min) / self.scale).round() as u8
            })
            .collect()
    }

    pub fn dequantize(&self, data: &[u8]) -> Vec<f32> {
        data.iter()
            .map(|&x| self.min + (x as f32) * self.scale)
            .collect()
    }

    pub fn l2_distance_sq(&self, a: &[u8], b: &[u8]) -> f32 {
        let sum_sq: u32 = a.iter().zip(b.iter())
            .map(|(&ai, &bi)| {
                let diff = ai as i32 - bi as i32;
                (diff * diff) as u32
            })
            .sum();
        (sum_sq as f32) * self.scale * self.scale
    }

    pub fn cosine_distance(&self, a: &[u8], b: &[u8]) -> f32 {
        let mut dot: f64 = 0.0;
        let mut norm_a: f64 = 0.0;
        let mut norm_b: f64 = 0.0;

        for (&ai, &bi) in a.iter().zip(b.iter()) {
            let val_a = self.min + (ai as f32) * self.scale;
            let val_b = self.min + (bi as f32) * self.scale;
            dot += (val_a * val_b) as f64;
            norm_a += (val_a * val_a) as f64;
            norm_b += (val_b * val_b) as f64;
        }

        if norm_a <= 0.0 || norm_b <= 0.0 {
            return 1.0;
        }

        let similarity = dot / (norm_a.sqrt() * norm_b.sqrt());
        (1.0 - similarity) as f32
    }
}
