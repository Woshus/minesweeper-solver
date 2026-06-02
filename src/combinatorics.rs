pub struct Combinatorics {
    ln_factorials: Vec<f64>,
}

impl Combinatorics {
    pub fn new(max: usize) -> Self {
        let mut ln_factorials = vec![0.0; max + 1];
        for i in 2..=max {
            ln_factorials[i] = ln_factorials[i - 1] + (i as f64).ln();
        }
        Self { ln_factorials }
    }

    pub fn ln_ncr(&self, n: usize, r: usize) -> f64 {
        if r > n {
            return f64::NEG_INFINITY;
        }
        if r == 0 || r == n {
            return 0.0;
        }

        self.ln_factorials[n] - self.ln_factorials[r] - self.ln_factorials[n - r]
    }
}
