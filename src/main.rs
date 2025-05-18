mod bigint_utils;
mod dh_exchange;
mod ec;
mod sign_ecdsa;
mod trg_enc;
use std::char;
use std::result;

use bigint_utils::FromHex;
use ec::EC;
use num_bigint::BigInt as bui;
use num_bigint::BigUint;
use num_bigint::Sign;
use sha2::{Digest, Sha256};

fn sha256(m: &[u8]) -> bui {
    let mut hasher = Sha256::new();
    hasher.update(m);
    bui::from_bytes_be(Sign::Plus, &hasher.finalize())
}

fn main() {
    let data = b"hello world";
    println!("{:?}", data);
    for i in data {
        println!("{}", *i as char);
    }

    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let bigint = sha256(data);

    // let bigint = BigUint::from_bytes_be(&result);
    println!("SHA-256 (bui hex): {}", bigint);
    println!("SHA-256 (hex): {:x}", result);

    // let hex = "0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551";
    // // let value: bui = bui::from_hex(hex).unwrap();
    // println!("Parsed: {}", value);
}
