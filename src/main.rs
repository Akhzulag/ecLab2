mod ec;
use ec::EC;

fn main() {
    for _ in 0..1000 {
        let (p, ec) = EC::gen_point_p256();
        println!("{:?}", p);
        println!("{}", ec.on_curve(&p));
    }
}
