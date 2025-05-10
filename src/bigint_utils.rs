use num_bigint::BigInt as bui;
use num_bigint::ParseBigIntError;
use num_traits::Num;

pub trait FromHex {
    fn from_hex(s: &str) -> Result<bui, ParseBigIntError>;
}

impl FromHex for bui {
    fn from_hex(s: &str) -> Result<bui, ParseBigIntError> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        bui::from_str_radix(s, 16)
    }
}
