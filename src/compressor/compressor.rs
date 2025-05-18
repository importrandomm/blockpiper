use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::Path;
use rayon::prelude::*;

use crate::grammar::Grammar;
use crate::grammar::grammar::Symbol;
use crate::ctw::Ctw;
use crate::arithmetic::{ArithmeticEncoder, ArithmeticDecoder};

const DEFAULT_BLOCK_SIZE: usize = 256 * 1024; // 256 KB

pub fn compress_file<P: AsRef<Path>>(input_path: P, output_path: P, block_size: Option<usize>) -> std::io::Result<()> {
    let block_size = block_size.unwrap_or(DEFAULT_BLOCK_SIZE);

    let mut input_file = File::open(input_path)?;
    let mut reader = BufReader::new(&mut input_file);

    let mut output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(&mut output_file);

    let mut blocks = Vec::new();
    let mut buffer = vec![0u8; block_size];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        blocks.push(buffer[..bytes_read].to_vec());
    }

    let compressed_blocks: Vec<(Vec<u8>, usize)> = blocks.par_iter().map(|block_data| {
        // Stage 1: Grammar-Based Modeling
        let mut grammar = Grammar::new();
        grammar.infer_grammar(block_data);
        let symbol_stream = serialize_grammar(&grammar);

        // Stage 2 & 3: CTW and Arithmetic Coding
        let mut ctw = Ctw::new();
        let mut encoder = ArithmeticEncoder::new();

        for &symbol in symbol_stream.iter() {
            let (cum, total) = ctw.get_cumulative(symbol);
            encoder.encode_symbol(symbol, (cum, total));
            ctw.process_symbol(symbol);
        }

        (encoder.finish(), block_data.len())
    }).collect();

    // Write compressed_blocks to the output file, including block sizes and original sizes
    for (compressed_block, orig_len) in compressed_blocks {
        let block_len = compressed_block.len() as u32;
        let orig_len = orig_len as u32;
        writer.write_all(&block_len.to_le_bytes())?;
        writer.write_all(&orig_len.to_le_bytes())?;
        writer.write_all(&compressed_block)?;
    }

    Ok(())
}

pub fn decompress_file<P: AsRef<Path>>(input_path: P, output_path: P) -> std::io::Result<()> {
    let mut input_file = File::open(input_path)?;
    let mut reader = BufReader::new(&mut input_file);
    let mut output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(&mut output_file);

    let mut block_len_buf = [0u8; 4];
    let mut orig_len_buf = [0u8; 4];
    while reader.read_exact(&mut block_len_buf).is_ok() {
        let block_len = u32::from_le_bytes(block_len_buf) as usize;
        reader.read_exact(&mut orig_len_buf)?;
        let orig_len = u32::from_le_bytes(orig_len_buf) as usize;
        let mut compressed_block = vec![0u8; block_len];
        reader.read_exact(&mut compressed_block)?;

        // Stage 2 & 3: Arithmetic Decoding and CTW
        let mut ctw = Ctw::new();
        let mut decoder = ArithmeticDecoder::new(compressed_block);
        let mut symbol_stream = Vec::new();
        for _ in 0..(orig_len * 8) { // upper bound, should be enough for grammar serialization
            let (cum, total) = ctw.get_cumulative(0);
            let symbol = decoder.decode_symbol((cum, total));
            symbol_stream.push(symbol);
            ctw.process_symbol(symbol);
            // Try to deserialize grammar and check if we have enough data
            if let Some(original_block) = deserialize_grammar(&symbol_stream) {
                if original_block.len() == orig_len {
                    writer.write_all(&original_block)?;
                    break;
                }
            }
        }
    }
    Ok(())
}

pub fn serialize_grammar(grammar: &Grammar) -> Vec<u8> {
    // Simple serialization: [num_rules][rule_id][rule_len][symbols...][sequence_len][sequence...]
    let mut out = Vec::new();
    out.extend(&(grammar.rules.len() as u32).to_le_bytes());
    for (&rule_id, expansion) in &grammar.rules {
        out.extend(&(rule_id as u32).to_le_bytes());
        out.extend(&(expansion.len() as u32).to_le_bytes());
        for symbol in expansion {
            match symbol {
                Symbol::Terminal(b) => {
                    out.push(0); // tag for terminal
                    out.push(*b);
                }
                Symbol::NonTerminal(id) => {
                    out.push(1); // tag for nonterminal
                    out.extend(&(*id as u32).to_le_bytes());
                }
            }
        }
    }
    out.extend(&(grammar.sequence.len() as u32).to_le_bytes());
    for symbol in &grammar.sequence {
        match symbol {
            Symbol::Terminal(b) => {
                out.push(0);
                out.push(*b);
            }
            Symbol::NonTerminal(id) => {
                out.push(1);
                out.extend(&(*id as u32).to_le_bytes());
            }
        }
    }
    out
}

fn deserialize_grammar(data: &[u8]) -> Option<Vec<u8>> {
    use std::collections::HashMap;
    let mut pos = 0;
    let read_u32 = |data: &[u8], pos: &mut usize| {
        if *pos + 4 > data.len() { return None; }
        let val = u32::from_le_bytes([data[*pos], data[*pos+1], data[*pos+2], data[*pos+3]]);
        *pos += 4;
        Some(val)
    };
    let read_symbol = |data: &[u8], pos: &mut usize| -> Option<Symbol> {
        if *pos >= data.len() { return None; }
        let tag = data[*pos];
        *pos += 1;
        match tag {
            0 => { // Terminal
                if *pos >= data.len() { return None; }
                let b = data[*pos];
                *pos += 1;
                Some(Symbol::Terminal(b))
            }
            1 => { // NonTerminal
                if *pos + 4 > data.len() { return None; }
                let id = u32::from_le_bytes([data[*pos], data[*pos+1], data[*pos+2], data[*pos+3]]) as usize;
                *pos += 4;
                Some(Symbol::NonTerminal(id))
            }
            _ => None,
        }
    };
    // Read rules
    let num_rules = read_u32(data, &mut pos)? as usize;
    let mut rules = HashMap::new();
    for _ in 0..num_rules {
        let rule_id = read_u32(data, &mut pos)? as usize;
        let rule_len = read_u32(data, &mut pos)? as usize;
        let mut expansion = Vec::new();
        for _ in 0..rule_len {
            expansion.push(read_symbol(data, &mut pos)?);
        }
        rules.insert(rule_id, expansion);
    }
    // Read sequence
    let seq_len = read_u32(data, &mut pos)? as usize;
    let mut sequence = Vec::new();
    for _ in 0..seq_len {
        sequence.push(read_symbol(data, &mut pos)?);
    }
    // Expand the sequence using the rules
    let mut output = Vec::new();
    fn expand(symbol: &Symbol, rules: &HashMap<usize, Vec<Symbol>>, out: &mut Vec<u8>) {
        match symbol {
            Symbol::Terminal(b) => out.push(*b),
            Symbol::NonTerminal(id) => {
                if let Some(exp) = rules.get(id) {
                    for s in exp {
                        expand(s, rules, out);
                    }
                }
            }
        }
    }
    for s in &sequence {
        expand(s, &rules, &mut output);
    }
    Some(output)
} 