use std::str::FromStr;

use crate::ec::{Point, EC};

use num_bigint::BigInt as bui;
use num_bigint::RandBigInt;
use num_bigint::ToBigInt;
use num_traits::Zero;

use num_integer::Integer;

struct ECDSA {
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

impl ECDSA {
    fn get_pk(&self) -> Result<&Point, String> {
        match &self.q_a {
            Some(point) => Ok(point),
            None => Err(String::from("You have not created public key")),
        }
    }

    fn init(ec: EC) -> ECDSA {
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
                let n: bui = self.ec.get_ref_q() - &1.to_bigint().unwrap();
                let h = sha256(m);
                loop {
                    let k = rng.gen_bigint_range(&(2).to_bigint().unwrap(), &n);

                    let kp = self.ec.scalar_mul(self.ec.get_ref_p(), &k);
                    let (x_1, _) = kp.get_xy(&self.ec);
                    let r = x_1 % &n;
                    if r != bui::zero() {
                        // extended_gcd(&self.q).
                        let k_rev = k.extended_gcd(&n).x; // тут має оберенене значення k
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
        let n: bui = self.ec.get_ref_q() - &1.to_bigint().unwrap();

        let s_rev = sign.1.extended_gcd(&n).x;

        let u1 = (&s_rev * h) % &n;
        let u2 = (&s_rev * &sign.0) % &n;
        let u1p = self.ec.scalar_mul(self.ec.get_ref_p(), &u1);
        let u2q_a = self.ec.scalar_mul(q_a, &u2);
        let (x_0, _) = self.ec.add(&u1p, &u2q_a).unwrap().get_xy(&self.ec);
        let v = x_0 % n;
        v == sign.0
    }
}
