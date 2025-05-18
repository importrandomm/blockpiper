use std::collections::HashMap;

const CTW_CONTEXT_LEN: usize = 4;

pub struct Ctw {
    context: Vec<u8>,
    tree: HashMap<Vec<u8>, [u32; 256]>,
}

impl Ctw {
    pub fn new() -> Self {
        Ctw {
            context: Vec::with_capacity(CTW_CONTEXT_LEN),
            tree: HashMap::new(),
        }
    }

    pub fn process_symbol(&mut self, symbol: u8) {
        let ctx = self.context.clone();
        let entry = self.tree.entry(ctx).or_insert([0; 256]);
        entry[symbol as usize] += 1;
        self.context.push(symbol);
        if self.context.len() > CTW_CONTEXT_LEN {
            self.context.remove(0);
        }
    }

    /// Returns (cumulative, total) for the symbol, for use with arithmetic coding
    pub fn get_cumulative(&self, symbol: u8) -> (u32, u32) {
        let ctx = &self.context;
        let counts = self.tree.get(ctx).unwrap_or(&[1; 256]); // Laplace smoothing
        let mut cumulative = 0;
        for i in 0..(symbol as usize) {
            cumulative += counts[i];
        }
        let total: u32 = counts.iter().sum();
        (cumulative, total.max(1))
    }
} 