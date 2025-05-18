use constriction::stream::{model::UniformModel, queue::{DefaultRangeEncoder, DefaultRangeDecoder}, Encode, Decode};
use bytemuck;

pub struct ArithmeticEncoder {
    encoder: DefaultRangeEncoder,
}

impl ArithmeticEncoder {
    pub fn new() -> Self {
        ArithmeticEncoder {
            encoder: DefaultRangeEncoder::new(),
        }
    }

    pub fn encode_symbol(&mut self, symbol: u8, _prob: (u32, u32)) {
        let model = UniformModel::<u8, 16>::new(255);
        self.encoder.encode_symbol(symbol, &model).unwrap();
    }

    pub fn finish(self) -> Vec<u8> {
        let compressed: Vec<u32> = self.encoder.into_compressed().unwrap();
        bytemuck::cast_slice(&compressed).to_vec()
    }
}

pub struct ArithmeticDecoder {
    decoder: DefaultRangeDecoder,
}

impl ArithmeticDecoder {
    pub fn new(encoded: Vec<u8>) -> Self {
        let compressed: Vec<u32> = bytemuck::cast_slice(&encoded).to_vec();
        ArithmeticDecoder {
            decoder: DefaultRangeDecoder::from_compressed(compressed).unwrap(),
        }
    }

    pub fn decode_symbol(&mut self, _prob: (u32, u32)) -> u8 {
        let model = UniformModel::<u8, 16>::new(255);
        self.decoder.decode_symbol(&model).unwrap()
    }
} 