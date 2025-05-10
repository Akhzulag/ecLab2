use std::str::FromStr;

use crate::ec::{Point, EC};

use num_bigint::BigInt as bui;
use num_bigint::RandBigInt;
use num_bigint::ToBigInt;

struct DH {
    ec: EC,
    d_a: Option<bui>,
}

impl DH {
    fn init(ec: EC) -> DH {
        let mut rng = rand::thread_rng();

        let d = rng.gen_bigint_range(
            &(2).to_bigint().unwrap(),
            &(ec.get_ref_q() - &1.to_bigint().unwrap()),
        );

        Self { ec, d_a: Some(d) }
    }

    fn send(&mut self) -> Point {
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
