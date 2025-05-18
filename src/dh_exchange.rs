use std::str::FromStr;

use crate::ec::{Point, EC};

use num_bigint::BigInt as bui;
use num_bigint::RandBigInt;
use num_bigint::ToBigInt;
use num_traits::One;

struct DH {
    ec: EC,
    d_a: Option<bui>,
}

impl DH {
    fn init(ec: EC) -> DH {
        let mut rng = rand::thread_rng();

        let d = rng.gen_bigint_range(&(2).to_bigint().unwrap(), ec.get_ref_n());

        Self { ec, d_a: Some(d) }
    }

    fn send(&self) -> Point {
        self.ec
            .scalar_mul(self.ec.get_ref_p(), self.d_a.as_ref().unwrap())
    }

    fn recieve(&self, q_b: &Point) -> Result<Point, String> {
        match &self.d_a {
            Some(d) => Ok(self.ec.scalar_mul(q_b, d)),
            None => Err(String::from("You don't have private parametrs")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bigint_utils::FromHex;
    use std::time::{Duration, Instant};

    #[test]
    fn test_DH() {
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

        let mut total = Duration::ZERO;
        let t = 100;
        for _ in 0..t {
            let start = Instant::now();
            let Alice = DH::init(ec_p256.clone());
            let Bob = DH::init(ec_p256.clone());

            let bob_sec = Alice.recieve(&Bob.send());
            let alice_sec = Bob.recieve(&Alice.send());

            total += start.elapsed();

            assert!(Point::cmp(
                &ec_p256.convert(&bob_sec.unwrap()).unwrap(),
                &ec_p256.convert(&alice_sec.unwrap()).unwrap()
            ));
        }

        println!("DH time: {:?}", total / t);
    }
}
