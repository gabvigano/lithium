pub const GLYPH_SIZE: usize = 8;
pub type Glyph8x8 = [u8; 8];

#[rustfmt::skip]
pub const GLYPH_A: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00111000,
    0b00000100,
    0b00111100,
    0b01000100,
    0b00111100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_B: Glyph8x8 = [
    0b00000000,
    0b01000000,
    0b01000000,
    0b01111000,
    0b01000100,
    0b01000100,
    0b01111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_C: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00111000,
    0b01000100,
    0b01000000,
    0b01000100,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_D: Glyph8x8 = [
    0b00000000,
    0b00000100,
    0b00000100,
    0b00111100,
    0b01000100,
    0b01000100,
    0b00111100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_E: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00111000,
    0b01000100,
    0b01111100,
    0b01000000,
    0b00111100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_F: Glyph8x8 = [
    0b00000000,
    0b00011100,
    0b00100000,
    0b01111000,
    0b00100000,
    0b00100000,
    0b00100000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_G: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00111100,
    0b01000100,
    0b00111100,
    0b00000100,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_H: Glyph8x8 = [
    0b00000000,
    0b01000000,
    0b01000000,
    0b01111000,
    0b01000100,
    0b01000100,
    0b01000100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_I: Glyph8x8 = [
    0b00000000,
    0b00010000,
    0b00000000,
    0b00110000,
    0b00010000,
    0b00010000,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_J: Glyph8x8 = [
    0b00000000,
    0b00001000,
    0b00000000,
    0b00011000,
    0b00001000,
    0b01001000,
    0b00110000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_K: Glyph8x8 = [
    0b00000000,
    0b01000000,
    0b01001000,
    0b01010000,
    0b01100000,
    0b01010000,
    0b01001000,

    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_L: Glyph8x8 = [
    0b00000000,
    0b00110000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_M: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01101000,
    0b01010100,
    0b01010100,
    0b01000100,
    0b01000100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_N: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01111000,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01000100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_O: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00111000,
    0b01000100,
    0b01000100,
    0b01000100,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_P: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01111000,
    0b01000100,
    0b01111000,
    0b01000000,
    0b01000000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_Q: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00111100,
    0b01000100,
    0b00111100,
    0b00000100,
    0b00000100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_R: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01011100,
    0b01100000,
    0b01000000,
    0b01000000,
    0b01000000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_S: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00111100,
    0b01000000,
    0b00111000,
    0b00000100,
    0b01111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_T: Glyph8x8 = [
    0b00000000,
    0b00100000,
    0b00100000,
    0b01111000,
    0b00100000,
    0b00100000,
    0b00011100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_U: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01001100,
    0b00110100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_V: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01000100,
    0b01000100,
    0b01000100,
    0b00101000,
    0b00010000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_W: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01000100,
    0b01000100,
    0b01010100,
    0b01010100,
    0b00101000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_X: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01000100,
    0b00101000,
    0b00010000,
    0b00101000,
    0b01000100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_Y: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01000100,
    0b01000100,
    0b00111100,
    0b00000100,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_Z: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01111100,
    0b00001000,
    0b00010000,
    0b00100000,
    0b01111100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_0: Glyph8x8 = [
    0b00000000,
    0b00111000,
    0b01000100,
    0b01001100,
    0b01010100,
    0b01100100,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_1: Glyph8x8 = [
    0b00000000,
    0b00010000,
    0b00110000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_2: Glyph8x8 = [
    0b00000000,
    0b00111000,
    0b01000100,
    0b00001000,
    0b00010000,
    0b00100000,
    0b01111100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_3: Glyph8x8 = [
    0b00000000,
    0b00111000,
    0b01000100,
    0b00011000,
    0b00000100,
    0b01000100,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_4: Glyph8x8 = [
    0b00000000,
    0b00001000,
    0b00011000,
    0b00101000,
    0b01001000,
    0b01111100,
    0b00001000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_5: Glyph8x8 = [
    0b00000000,
    0b01111100,
    0b01000000,
    0b01111000,
    0b00000100,
    0b01000100,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_6: Glyph8x8 = [
    0b00000000,
    0b00111000,
    0b01000000,
    0b01111000,
    0b01000100,
    0b01000100,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_7: Glyph8x8 = [
    0b00000000,
    0b01111100,
    0b00000100,
    0b00001000,
    0b00010000,
    0b00100000,
    0b00100000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_8: Glyph8x8 = [
    0b00000000,
    0b00111000,
    0b01000100,
    0b00111000,
    0b01000100,
    0b01000100,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_9: Glyph8x8 = [
    0b00000000,
    0b00111000,
    0b01000100,
    0b01000100,
    0b00111100,
    0b00000100,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_LEFT_PAREN: Glyph8x8 = [
    0b00000000,
    0b00001100,
    0b00010000,
    0b00100000,
    0b00100000,
    0b00010000,
    0b00001100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_RIGHT_PAREN: Glyph8x8 = [
    0b00000000,
    0b00110000,
    0b00001000,
    0b00000100,
    0b00000100,
    0b00001000,
    0b00110000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_LEFT_BRACKET: Glyph8x8 = [
    0b00000000,
    0b00111000,
    0b00100000,
    0b00100000,
    0b00100000,
    0b00100000,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_RIGHT_BRACKET: Glyph8x8 = [
    0b00000000,
    0b00111000,
    0b00001000,
    0b00001000,
    0b00001000,
    0b00001000,
    0b00111000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_LEFT_BRACE: Glyph8x8 = [
    0b00000000,
    0b00001100,
    0b00010000,
    0b00110000,
    0b00010000,
    0b00010000,
    0b00001100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_RIGHT_BRACE: Glyph8x8 = [
    0b00000000,
    0b00110000,
    0b00001000,
    0b00001100,
    0b00001000,
    0b00001000,
    0b00110000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_COLON: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00010000,
    0b00000000,
    0b00000000,
    0b00010000,
    0b00000000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_SEMICOLON: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00010000,
    0b00000000,
    0b00000000,
    0b00010000,
    0b00010000,
    0b00100000,
];

#[rustfmt::skip]
pub const GLYPH_MINUS: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00000000,
    0b00111000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_ASTERISK: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01010100,
    0b00111000,
    0b01111100,
    0b00111000,
    0b01010100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_PLUS: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00010000,
    0b00010000,
    0b01111100,
    0b00010000,
    0b00010000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_EQUALS: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b01111100,
    0b00000000,
    0b01111100,
    0b00000000,
    0b00000000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_PIPE: Glyph8x8 = [
    0b00000000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_SLASH: Glyph8x8 = [
    0b00000000,
    0b00000100,
    0b00001000,
    0b00010000,
    0b00100000,
    0b01000000,
    0b00000000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_BACKSLASH: Glyph8x8 = [
    0b00000000,
    0b01000000,
    0b00100000,
    0b00010000,
    0b00001000,
    0b00000100,
    0b00000000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_DOT: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00011000,
    0b00011000,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_COMMA: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00011000,
    0b00011000,
    0b00110000,
];

#[rustfmt::skip]
pub const GLYPH_LESS_THAN: Glyph8x8 = [
    0b00000000,
    0b00000100,
    0b00001000,
    0b00010000,
    0b00100000,
    0b00010000,
    0b00001000,
    0b00000100,
];

#[rustfmt::skip]
pub const GLYPH_GREATER_THAN: Glyph8x8 = [
    0b00000000,
    0b00100000,
    0b00010000,
    0b00001000,
    0b00000100,
    0b00001000,
    0b00010000,
    0b00100000,
];

#[rustfmt::skip]
pub const GLYPH_UNDERSCORE: Glyph8x8 = [
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b01111100,
    0b00000000,
];

#[rustfmt::skip]
pub const GLYPH_SPACE: Glyph8x8 = [0; 8];

#[rustfmt::skip]
pub const GLYPH_UNKNOWN: Glyph8x8 = [
    0b00000000,
    0b11111110,
    0b10000010,
    0b10101010,
    0b10010010,
    0b10101010,
    0b10000010,
    0b11111110,
];

pub fn match_glyph(character: u8) -> &'static Glyph8x8 {
    match character {
        97 => &GLYPH_A,
        98 => &GLYPH_B,
        99 => &GLYPH_C,
        100 => &GLYPH_D,
        101 => &GLYPH_E,
        102 => &GLYPH_F,
        103 => &GLYPH_G,
        104 => &GLYPH_H,
        105 => &GLYPH_I,
        106 => &GLYPH_J,
        107 => &GLYPH_K,
        108 => &GLYPH_L,
        109 => &GLYPH_M,
        110 => &GLYPH_N,
        111 => &GLYPH_O,
        112 => &GLYPH_P,
        113 => &GLYPH_Q,
        114 => &GLYPH_R,
        115 => &GLYPH_S,
        116 => &GLYPH_T,
        117 => &GLYPH_U,
        118 => &GLYPH_V,
        119 => &GLYPH_W,
        120 => &GLYPH_X,
        121 => &GLYPH_Y,
        122 => &GLYPH_Z,

        48 => &GLYPH_0,
        49 => &GLYPH_1,
        50 => &GLYPH_2,
        51 => &GLYPH_3,
        52 => &GLYPH_4,
        53 => &GLYPH_5,
        54 => &GLYPH_6,
        55 => &GLYPH_7,
        56 => &GLYPH_8,
        57 => &GLYPH_9,

        40 => &GLYPH_LEFT_PAREN,
        41 => &GLYPH_RIGHT_PAREN,
        91 => &GLYPH_LEFT_BRACKET,
        93 => &GLYPH_RIGHT_BRACKET,
        123 => &GLYPH_LEFT_BRACE,
        125 => &GLYPH_RIGHT_BRACE,
        58 => &GLYPH_COLON,
        59 => &GLYPH_SEMICOLON,
        45 => &GLYPH_MINUS,
        42 => &GLYPH_ASTERISK,
        43 => &GLYPH_PLUS,
        61 => &GLYPH_EQUALS,
        124 => &GLYPH_PIPE,
        47 => &GLYPH_SLASH,
        92 => &GLYPH_BACKSLASH,
        46 => &GLYPH_DOT,
        44 => &GLYPH_COMMA,
        60 => &GLYPH_LESS_THAN,
        62 => &GLYPH_GREATER_THAN,
        95 => &GLYPH_UNDERSCORE,
        32 => &GLYPH_SPACE,

        _ => &GLYPH_UNKNOWN,
    }
}
