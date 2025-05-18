use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    Terminal(u8),
    NonTerminal(usize),
}

type Digram = (Symbol, Symbol);

#[derive(Debug, Clone)]
pub struct Grammar {
    pub rules: HashMap<usize, Vec<Symbol>>,
    pub next_nonterminal_id: usize,
    pub sequence: Vec<Symbol>,
}

impl Grammar {
    pub fn new() -> Self {
        Grammar {
            rules: HashMap::new(),
            next_nonterminal_id: 0,
            sequence: Vec::new(),
        }
    }

    /// Robust Sequitur implementation
    pub fn infer_grammar(&mut self, data: &[u8]) {
        self.sequence = data.iter().map(|&b| Symbol::Terminal(b)).collect();
        let mut digram_map: HashMap<Digram, Vec<usize>> = HashMap::new();
        let mut rule_usage: HashMap<usize, usize> = HashMap::new();
        let mut i = 0;
        while i + 1 < self.sequence.len() {
            let digram = (self.sequence[i].clone(), self.sequence[i + 1].clone());
            let entry = digram_map.entry(digram.clone()).or_insert_with(Vec::new);
            entry.push(i);
            if entry.len() == 2 {
                // Digram repeats, create or use rule
                let rule_id = self.find_or_create_rule(&digram, &mut rule_usage);
                // Replace all occurrences of this digram in the sequence
                self.replace_all_digrams(&digram, rule_id, &mut digram_map);
                // Enforce rule utility
                self.enforce_rule_utility(&mut rule_usage);
                // Restart scan
                i = 0;
                digram_map.clear();
                continue;
            }
            i += 1;
        }
    }

    fn find_or_create_rule(&mut self, digram: &Digram, rule_usage: &mut HashMap<usize, usize>) -> usize {
        // Check if a rule already exists for this digram
        for (&rid, exp) in &self.rules {
            if exp.len() == 2 && exp[0] == digram.0 && exp[1] == digram.1 {
                return rid;
            }
        }
        // Create new rule
        let rule_id = self.next_nonterminal_id;
        self.next_nonterminal_id += 1;
        self.rules.insert(rule_id, vec![digram.0.clone(), digram.1.clone()]);
        rule_usage.insert(rule_id, 0);
        rule_id
    }

    fn replace_all_digrams(&mut self, digram: &Digram, rule_id: usize, digram_map: &mut HashMap<Digram, Vec<usize>>) {
        let mut i = 0;
        while i + 1 < self.sequence.len() {
            if self.sequence[i] == digram.0 && self.sequence[i + 1] == digram.1 {
                self.sequence.remove(i);
                self.sequence[i] = Symbol::NonTerminal(rule_id);
                // After replacement, don't increment i (check for overlapping digrams)
            } else {
                i += 1;
            }
        }
    }

    fn enforce_rule_utility(&mut self, rule_usage: &mut HashMap<usize, usize>) {
        // Count rule usage in sequence
        rule_usage.clear();
        for s in &self.sequence {
            if let Symbol::NonTerminal(id) = s {
                *rule_usage.entry(*id).or_insert(0) += 1;
            }
        }
        // Remove rules used only once
        let to_remove: Vec<usize> = rule_usage.iter().filter(|(_, &count)| count == 1).map(|(&id, _)| id).collect();
        for rid in to_remove {
            if let Some(exp) = self.rules.remove(&rid) {
                // Replace the single occurrence in the sequence
                let mut i = 0;
                while i < self.sequence.len() {
                    if self.sequence[i] == Symbol::NonTerminal(rid) {
                        self.sequence.remove(i);
                        for (j, s) in exp.iter().enumerate() {
                            self.sequence.insert(i + j, s.clone());
                        }
                        break;
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }
}

// (Keep only one set of tests at the end)