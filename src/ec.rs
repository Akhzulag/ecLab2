#![allow(unused)]

use crate::bigint_utils::FromHex;
use num_bigint::BigInt as bui;
use num_bigint::RandBigInt;
use num_integer::Integer;
use num_traits::{One, Zero};
use std::{convert, str::FromStr};
#[derive(Clone, Debug)]
pub struct Point {
    x: Option<bui>,
    y: Option<bui>,
    z: Option<bui>,
}

pub struct EC {
    a: bui,
    b: bui,
    q: bui,
    n: Option<bui>,
    p: Option<Point>,
}

impl Point {
    pub fn new(x: bui, y: bui, z: Option<bui>) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
            z,
        }
    }

    fn cmp(&self, p: &Point) -> bool {
        if self.x == p.x && self.y == p.y && self.z == p.z {
            return true;
        }
        false
    }

    pub fn get_xy(&self, ec: &EC) -> (bui, bui) {
        match self.z.as_ref() {
            Some(z) => {
                let point = ec.convert(self).unwrap();
                (point.x.unwrap(), point.y.unwrap())
            }
            None => (
                self.x.as_ref().unwrap().clone(),
                self.y.as_ref().unwrap().clone(),
            ),
        }
    }
}

impl EC {
    pub fn new(a: bui, b: bui, q: bui, n: Option<bui>, p: Option<Point>) -> Self {
        Self { a, b, q, n, p }
    }

    pub fn get_ref_q(&self) -> &bui {
        &self.q
    }

    pub fn get_ref_p(&self) -> &Point {
        self.p
            .as_ref()
            .expect("eliptic curve does not contain generator point")
    }

    pub fn on_curve(&self, p: &Point) -> bool {
        match (&p.x, &p.y, &p.z) {
            (Some(x), Some(y), None) => {
                let left = y.modpow(&bui::from_str("2").unwrap(), &self.q);
                let right = (x.pow(3) + &self.a * x + &self.b) % &self.q;
                right == left
            }
            (_, _, _) => self.on_curve(&self.convert(p).expect("problem on curve")),
        }
    }

    pub fn convert(&self, p: &Point) -> Result<Point, String> {
        match (&p.x, &p.y, &p.z) {
            (Some(x), Some(y), Some(z)) => {
                if *z == bui::zero() {
                    return Ok(Point::new(bui::zero(), bui::one(), Some(bui::zero())));
                }
                let z1 = (z.extended_gcd(&self.q).x + &self.q) % &self.q;
                Ok(Point::new((x * &z1) % &self.q, (y * z1) % &self.q, None))
            }
            (Some(x), Some(y), None) => Ok(Point::new(x.clone(), y.clone(), Some(bui::one()))),
            (_, _, _) => Err(String::from("помилка")),
        }
    }

    pub fn double(&self, p: &Point) -> Result<Point, String> {
        match (&p.x, &p.y, &p.z) {
            (Some(x), Some(y), Some(z)) => {
                let o_e = Point::new(bui::zero(), bui::one(), Some(bui::zero()));

                if p.cmp(&o_e) {
                    return Ok(o_e);
                }

                if *y == bui::zero() {
                    return Ok(o_e);
                };

                let w = &self.a * z.pow(2) + 3u32 * x.pow(2);
                let s = y * z;
                let b = x * y * &s;
                let h = w.pow(2) - 8u32 * &b;
                let x_ = 2u32 * &h * &s;
                let y_ = w * (4u32 * b - h) - 8u32 * y.pow(2) * s.pow(2);
                let z_ = 8u32 * s.pow(3);

                Ok(Point::new(
                    modulo(&x_, &self.q),
                    modulo(&y_, &self.q),
                    Some(modulo(&z_, &self.q)),
                ))
            }
            (_, _, _) => Err(String::from("помилка")),
        }
    }

    pub fn add(&self, p1: &Point, p2: &Point) -> Result<Point, String> {
        match (&p1.z, &p2.z) {
            (Some(z1), Some(z2)) => {
                let o_e = Point::new(bui::zero(), bui::one(), Some(bui::zero()));
                if p1.cmp(&o_e) {
                    // println!("p2: {:?}", p2);
                    return Ok(p2.clone());
                } else if p2.cmp(&o_e) {
                    return Ok(p1.clone());
                }
                let u1 = p2.y.as_ref().unwrap() * z1;
                let u2 = p1.y.as_ref().unwrap() * z2;
                let v1 = p2.x.as_ref().unwrap() * z1;
                let v2 = p1.x.as_ref().unwrap() * z2;
                if v1 == v2 {
                    if u1 != u2 {
                        return Ok(o_e);
                    } else {
                        return self.double(p1);
                    }
                }

                let u = u1 - &u2;
                let v = modulo(&(v1 - &v2), &self.q);

                // println!("v: {v}");
                if v == bui::zero() {
                    return Ok(o_e);
                }
                let w = z1 * z2;
                let a = u.pow(2) * &w - v.pow(3) - 2u32 * v.pow(2) * &v2;
                let x3 = &v * &a;
                let y3 = &u * (v.pow(2) * &v2 - a) - v.pow(3) * u2;
                let z3 = v.pow(3) * &w;

                Ok(Point::new(
                    modulo(&x3, &self.q),
                    modulo(&y3, &self.q),
                    Some(modulo(&z3, &self.q)),
                ))
            }

            (None, None) => Err(String::from("помилка")),
            (_, None) => Err(String::from("помилка")),
            (None, _) => Err(String::from("помилка")),
        }
    }

    pub fn scalar_mul(&self, p: &Point, k: &bui) -> Point {
        let mut r_0 = Point::new(bui::zero(), bui::one(), Some(bui::zero()));
        let mut r_1 = p.clone();
        let bitstring = format!("{:b}", k);
        let bitstring = bitstring.chars();
        for i in bitstring {
            if i == '0' {
                r_1 = self.add(&r_0, &r_1).unwrap();
                r_0 = self.double(&r_0).unwrap();
            } else {
                r_0 = self.add(&r_0, &r_1).unwrap();
                r_1 = self.double(&r_1).unwrap();
            }
        }

        r_0
    }

    pub fn gen_point_p256() -> (Point, EC) {
        let ec = EC::new(
            bui::from_str(
                "115792089210356248762697446949407573530086143415290314195533631308867097853948",
            )
            .unwrap(),
            bui::from_str(
                "41058363725152142129326129780047268409114441015993725554835256314039467401291",
            )
            .unwrap(),
            bui::from_str(
                "115792089210356248762697446949407573530086143415290314195533631308867097853951",
            )
            .unwrap(),
            Some(bui::from_str(
                "115792089210356248762697446949407573529996955224135760342422259061068512044369",
            )
            .unwrap()),
            None,
        );

        let mut rng = rand::thread_rng();
        loop {
            let x_0 = rng.gen_bigint_range(&bui::one(), &ec.q);
            let x = (x_0.pow(3) + &ec.a * &x_0 + &ec.b) % &ec.q;

            if legendre_symbol(&x, &ec.q) == bui::one() {
                let y = solve(&x, &ec.q);
                break (Point::new(x_0, y, Some(bui::one())), ec);
            }
        }
    }
}

fn modulo(a: &bui, n: &bui) -> bui {
    ((a % n) + n) % n
}

use num_bigint::ToBigInt;

fn legendre_symbol(a: &bui, p: &bui) -> bui {
    let pow = p - &bui::one();
    let pow: bui = pow / 2;
    let r = a.modpow(&pow, p);
    if r == p - 1 {
        return -1.to_bigint().unwrap();
    }

    r
}

fn solve(x: &bui, p: &bui) -> bui {
    let mut q = p - 1;
    let mut s = 0;
    while (&q & bui::one()) != bui::one() {
        q >>= 1;
        s += 1;
    }
    if s == 1 {
        return x.modpow(&((p + 1) >> 2), p);
    }

    let mut rng = rand::thread_rng();
    let z = loop {
        let k: bui = rng.gen_bigint_range(&bui::one(), &q);
        if legendre_symbol(&k, p) == -1.to_bigint().unwrap() {
            break k;
        }
    };

    let mut c = z.modpow(&q, p);

    let mut r = x.modpow(&((&q + 1) >> 1), p);

    let mut t = x.modpow(&q, p);

    let mut m = s;

    loop {
        if t == bui::one() {
            break r;
        }

        let mut i = 0;

        let i = loop {
            if t.modpow(&(2 >> i).to_bigint().unwrap(), p) == bui::one() {
                break i;
            }
            i += 1;
        };

        let b = c.modpow(&((2 >> (m - i - 1)).to_bigint().unwrap()), p);

        r = (r * &b) % p;
        t = (t * b.pow(2)) % p;
        c = b.pow(2) % p;
        m = i;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt as bui;

    #[test]
    fn test_point_cmp_equal() {
        let (p1, _) = EC::gen_point_p256();
        let p2 = p1.clone();

        assert!(p1.cmp(&p2));
    }

    #[test]
    fn test_ec_on_curve() {
        for _ in 0..1000 {
            let (p, ec) = EC::gen_point_p256();
            println!("{:?}", p);
            assert!(ec.on_curve(&p));
        }
    }

    #[test]
    fn test_ec_on_curve2() {
        for _ in 0..1000 {
            let (p, ec) = EC::gen_point_p256();
            let p = ec.double(&p).unwrap();
            println!("{:?}", p);
            assert!(ec.on_curve(&p));
        }
    }

    #[test]
    fn test_ec_add_double() {
        for _ in 0..1000 {
            let (p1, ec) = EC::gen_point_p256();

            let result_add = ec.add(&p1, &p1).unwrap();
            let result_double = ec.double(&p1).unwrap();
            assert!(result_add.cmp(&result_double));
        }
    }

    #[test]
    fn test_scalar_mul() {
        for _ in 0..100 {
            let (p, ec) = EC::gen_point_p256();
            let n = bui::from_str(
                "115792089210356248762697446949407573529996955224135760342422259061068512044369",
            )
            .unwrap();

            let o_e = Point::new(bui::zero(), bui::one(), Some(bui::zero()));
            let res = ec.scalar_mul(&p, ec.n.as_ref().unwrap());
            assert!(res.cmp(&o_e));
        }
    }
}
