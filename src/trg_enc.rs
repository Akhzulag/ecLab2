#![allow(unused)]
use crate::ec::{Point, EC};

use num_bigint::BigInt as bui;
use num_bigint::RandBigInt;
use num_bigint::ToBigInt;

struct TrgEnc {
    ec: EC,
    q_a: Option<Point>,
    e_a: Option<bui>,
}

fn wrap_dec(x: &bui, k: &bui) -> bui {
    x ^ k
}

impl TrgEnc {
    fn get_ref_q_a(&self) -> &Point {
        self.q_a.as_ref().expect("Dont have q_a")
    }

    fn init(ec: EC) -> TrgEnc {
        let mut rng = rand::thread_rng();

        let e_a = rng.gen_bigint_range(&(2).to_bigint().unwrap(), ec.get_ref_n());

        let q_a = ec.scalar_mul(ec.get_ref_p(), &e_a);

        Self {
            ec,
            q_a: Some(q_a),
            e_a: Some(e_a),
        }
    }

    fn enc(&self, m: &bui, q_b: &Point) -> (bui, bui) {
        let ec = &self.ec;
        let p = ec.get_ref_p();
        let e_a = self.e_a.as_ref().unwrap();
        let mut rng = rand::thread_rng();

        let k = rng.gen_bigint(256);
        let c_m = wrap_dec(m, &k);

        let (s_x, _) = ec.scalar_mul(q_b, e_a).get_xy(ec);
        let c_k = wrap_dec(&k, &s_x);

        (c_k, c_m)
    }

    fn dec(&self, q_a: &Point, c_k: &bui, c_m: &bui) -> bui {
        let d_b = self.e_a.as_ref().unwrap();
        let ec = &self.ec;

        let (s_x, _) = ec.scalar_mul(q_a, d_b).get_xy(ec);

        let k = wrap_dec(c_k, &s_x);
        wrap_dec(c_m, &k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bigint_utils::FromHex;
    use num_bigint::ToBigInt;
    use num_traits::{One, Zero};
    use std::time::{Duration, Instant};

    #[test]
    fn test_trg_enc_dec() {
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

        let mut total_enc = Duration::ZERO;
        let mut total_dec = Duration::ZERO;
        let t = 100;

        for _ in 0..t {
            let Bob = TrgEnc::init(ec_p256.clone());
            let Alice = TrgEnc::init(ec_p256.clone());

            let mut rng = rand::thread_rng();
            let message = rng.gen_bigint(256);

            let q_b = Alice.get_ref_q_a();

            let start = Instant::now();
            let (c_k, c_m) = Bob.enc(&message, q_b);

            total_enc += start.elapsed();

            let q_a = Bob.get_ref_q_a();

            let start = Instant::now();

            let decrypted = Alice.dec(q_a, &c_k, &c_m);

            total_dec += start.elapsed();

            assert_eq!(decrypted, message);
        }

        println!("time enc: {:?}", total_enc / t);
        println!("time dec: {:?}", total_dec / t);
    }
}
