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
    fn init(ec: EC) -> TrgEnc {
        let mut rng = rand::thread_rng();

        let e_a = rng.gen_bigint_range(
            &(2).to_bigint().unwrap(),
            &(ec.get_ref_q() - &1.to_bigint().unwrap()),
        );

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
