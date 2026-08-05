#[path = "../src/math.rs"]
mod math;

use math::PkmnRational;

#[test]
fn test_pow_operator_standalone() {
    let r = PkmnRational::new(2, 3);
    assert_eq!(r.pow(3), PkmnRational::new(8, 27));
    assert_eq!(r ^ 3, PkmnRational::new(8, 27));
}
