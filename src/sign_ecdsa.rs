#![allow(unused)]

use std::str::FromStr;

use crate::ec::{Point, EC};

use num_bigint::BigInt as bui;
use num_bigint::RandBigInt;
use num_bigint::ToBigInt;
use num_traits::Zero;

use num_integer::Integer;

struct Ecdsa {
    ec: EC,
    q_a: Option<Point>,
    d_a: Option<bui>,
}

use num_bigint::Sign;
use sha2::{Digest, Sha256};

fn sha256(m: &[u8]) -> bui {
    let mut hasher = Sha256::new();
    hasher.update(m);
    bui::from_bytes_be(Sign::Plus, &hasher.finalize())
}

impl Ecdsa {
    fn get_pk(&self) -> Result<&Point, String> {
        match &self.q_a {
            Some(point) => Ok(point),
            None => Err(String::from("You have not created public key")),
        }
    }

    fn init(ec: EC) -> Ecdsa {
        let mut rng = rand::thread_rng();

        let d_a = rng.gen_bigint_range(
            &(2).to_bigint().unwrap(),
            &(ec.get_ref_q() - &1.to_bigint().unwrap()),
        );

        let q_a = ec.scalar_mul(ec.get_ref_p(), &d_a);

        Self {
            ec,
            q_a: Some(q_a),
            d_a: Some(d_a),
        }
    }

    fn sign(&mut self, m: &[u8]) -> Result<(bui, bui), String> {
        match &self.d_a {
            Some(d_a) => {
                let mut rng = rand::thread_rng();
                let n: &bui = self.ec.get_ref_n();
                let h: bui = sha256(m);
                loop {
                    let k = rng.gen_bigint_range(&(2).to_bigint().unwrap(), n);

                    let kp = self.ec.scalar_mul(self.ec.get_ref_p(), &k);
                    let (x_1, _) = kp.get_xy(&self.ec);
                    let r = x_1 % n;
                    if r != bui::zero() {
                        let k_rev = k.extended_gcd(n).x;
                        let k_rev = modulo(&k_rev, n);
                        let s = (k_rev * (h + d_a * &r)) % n;
                        break Ok((r, s));
                    }
                }
            }
            None => Err(String::from("You have not created public key")),
        }
    }

    fn verify(&self, m: &[u8], sign: &(bui, bui), q_a: &Point) -> bool {
        let h = sha256(m);
        let n = self.ec.get_ref_n();

        let s_rev = sign.1.extended_gcd(n).x;
        let s_rev = modulo(&s_rev, n);
        let u1 = (&s_rev * h) % n;
        let u2 = (&s_rev * &sign.0) % n;
        let u1p = self.ec.scalar_mul(self.ec.get_ref_p(), &u1);
        let u2q_a = self.ec.scalar_mul(q_a, &u2);
        let (x_0, _) = self.ec.add(&u1p, &u2q_a).unwrap().get_xy(&self.ec);
        let v = x_0 % n;
        v == sign.0
    }
}

fn modulo(a: &bui, n: &bui) -> bui {
    ((a % n) + n) % n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bigint_utils::FromHex;
    use num_traits::One;
    use rand::Rng;

    use std::time::{Duration, Instant};
    #[test]
    fn test_sign() {
        let p = Point::new(
            bui::from_hex("6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296")
                .unwrap(),
            bui::from_hex("4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5")
                .unwrap(),
            Some(bui::one()),
        );
        let a = bui::from_hex("ffffffff00000001000000000000000000000000fffffffffffffffffffffffc")
            .unwrap();
        let b = bui::from_hex("5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b")
            .unwrap();
        let q = bui::from_hex("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff")
            .unwrap();
        let n = bui::from_hex("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551")
            .unwrap();
        let ec_p256 = EC::new(a, b, q, Some(n), Some(p.clone()));

        let mut total_sign = Duration::ZERO;
        let mut total_verf = Duration::ZERO;
        let t = 100;
        for _ in 0..t {
            let mut ecdsa = Ecdsa::init(ec_p256.clone());

            let mut rng = rand::thread_rng();
            let message: [u8; 32] = rng.gen();

            let start = Instant::now();

            let signature = ecdsa.sign(&message);

            let elapsed = start.elapsed();

            total_sign += elapsed;

            assert!(signature.is_ok(), "Не вдалося створити підпис");

            let (r, s) = signature.unwrap();

            let start = Instant::now();

            let pk = ecdsa.get_pk().unwrap();

            let elapsed = start.elapsed();

            total_verf += elapsed;

            let is_valid = ecdsa.verify(&message, &(r, s), pk);

            assert!(is_valid, "Підпис не пройшов перевірку");
        }

        println!("time sign: {:?}", total_sign / t);
        println!("time verf: {:?}", total_verf / t);
    }
}
