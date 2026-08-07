use rand::rngs::ThreadRng;

/// Basic functions

/// Multiplies an integer by a float and floors the result to the nearest integer.
/// Used to calculate many stat modifications and damage rolls
pub fn mult_and_round(a: i32, b: f64) -> i32 {
    (a as f64 * b).floor() as i32
}

pub fn div_and_floor(a: i32, b: i32) -> i32 {
    (a as f64 / b as f64).floor() as i32
}

#[derive(Copy, Clone, Debug)]
pub struct PkmnRational {
    pub numer: i32,
    pub demon: u32,
}

#[allow(non_snake_case)]
impl PkmnRational {

    const REDUCE_MIN:u32 = 10_000;
    
    pub fn ONE() -> PkmnRational {
        return PkmnRational { numer: 1, demon: 1 }
    }

    pub fn ZERO() -> PkmnRational {
        return PkmnRational { numer: 0, demon: 1 }
    }

    pub fn float(self) -> f64 {
        return self.numer as f64 / self.demon as f64
    }

    pub fn new(numer:i32, demon:u32) -> PkmnRational{
        return PkmnRational{
            numer: numer,
            demon: demon
        }
    }

    // Return fraction out of 100
    pub fn pct(numer:i32) -> PkmnRational {
        return PkmnRational { numer, demon: 100 }
    }

    /// Hacky convert to fraction
    pub fn from_float(float: f64) -> PkmnRational {
        let precision: u32 = 1_000_000;
        let numer = (float * precision as f64).round() as i32;
        let mut r = PkmnRational { numer, demon: precision };
        r.reduce();
        r
    }

    pub fn pow(self, exp: u32) -> PkmnRational {
        let mut out = PkmnRational::ONE();
        for _ in 0..exp {
            out = out * self;
        }
        out
    }
    
    // Find GCD and reduce numer/demon if needed
    fn reduce(&mut self) {
        // if self.demon > PkmnRational::REDUCE_MIN {
            // GCD algo
            let mut large: u32 = self.demon.max(self.numer.abs() as u32);
            let mut small:u32;
            if self.demon == large {
                small = self.numer as u32;
            } else {
                small = self.demon;
            }

            // let mut remain = 1;
            while small != 0 {
                let remain = large % small;
                if remain == 0 {
                    break
                }
                // remain = large;
                large = small;
                small = remain;
            }

            // once GCD is found, divide
            self.numer /= small as i32;
            self.demon /= small;
        // }
    }

}

impl core::fmt::Display for PkmnRational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pct = format!("{:04.2}", 
            (self.numer as f32)/ (self.demon as f32));
        write!(f, "{}/{} [{pct}%]", self.numer, self.demon)
    }
}

impl std::ops::Add for PkmnRational {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        return Self { numer: 
            rhs.numer * self.demon as i32 + self.numer * rhs.demon as i32, 
            demon: rhs.demon * self.demon 
        };
    }
}

impl std::ops::AddAssign for PkmnRational {

    fn add_assign(&mut self, rhs: Self) {
        self.numer = rhs.numer * self.demon as i32 + self.numer * rhs.demon as i32;
        self.demon *= rhs.demon;
        if self.demon > PkmnRational::REDUCE_MIN {
            self.reduce();
        }
    }
}

impl PartialEq for PkmnRational {
    fn eq(&self, other: &Self) -> bool {
        if self.demon == other.demon {
            return self.numer == other.numer;
        }
        self.numer * other.demon as i32 == other.numer * self.demon as i32
    }
}

impl std::ops::Sub for PkmnRational {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        return Self { numer: 
            self.numer * rhs.demon as i32 - rhs.numer * self.demon as i32, 
            demon: rhs.demon * self.demon 
        };
    }
}

impl std::ops::Mul<i32> for PkmnRational {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self {
        Self { numer: self.numer * rhs, demon: self.demon }
    }
}

impl std::ops::Mul<PkmnRational> for PkmnRational {
    type Output = Self;

    fn mul(self, rhs: PkmnRational) -> Self {
        Self { numer: self.numer * rhs.numer, demon: self.demon * rhs.demon }
    }
}

impl std::ops::MulAssign<PkmnRational> for PkmnRational {
    fn mul_assign(&mut self, rhs: PkmnRational) {
        self.numer *= rhs.numer;
        self.demon *= rhs.demon;
        if self.demon > PkmnRational::REDUCE_MIN {
            self.reduce();
        }
    }
}

impl std::ops::BitXor<u32> for PkmnRational {
    type Output = Self;

    fn bitxor(self, rhs: u32) -> Self {
        self.pow(rhs)
    }
}

impl std::iter::Product for PkmnRational {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(PkmnRational::ONE(), |mut acc, v| {
            acc *= v;
            acc
        })
    }
}

impl<'a> std::iter::Product<&'a PkmnRational> for PkmnRational {
    fn product<I: Iterator<Item = &'a PkmnRational>>(iter: I) -> Self {
        iter.fold(PkmnRational::ONE(), |mut acc, v| {
            acc *= *v;
            acc
        })
    }
}

/// return a generator with all probablities distributions
pub fn gen_power_set(probs:Vec<PkmnRational>) -> Vec<PkmnRational> {

    let inverse_probs:Vec<PkmnRational> = probs.iter().map(
        |prob| PkmnRational::ONE() - *prob
    ).collect();

    // TODO: Make this an generator with internal state
    /*
    See
        fn counter(start: i32, end: i32) -> impl Iterator<Item = i32> {
        let mut cur = start;
        std::iter::from_fn(move || {
            if cur <= end {
                let out = cur;
                cur += 1;
                Some(out)
            } else {
                None
            }
        })
    }
     */
    
    let power_set_total = 0b1 << probs.len();
    let mut result_probs:Vec<PkmnRational> = vec![];
    for bin_comb in 0..power_set_total {
        let mut curr_prob = PkmnRational::ONE();
        for idx in 0..probs.len() {
            let digit = (bin_comb >> idx) & 0b1;
            let mult_prob:PkmnRational = if digit == 0 {
                inverse_probs[idx]
            } else {
                probs[idx]
            };
            curr_prob *= mult_prob;
        }
        result_probs.push(curr_prob);
    }
    // calc each probably up to the power_set total
    result_probs
}


// TODO: Use this rng thread 
pub fn getRNGThread() -> ThreadRng {
    return rand::rng();
}

/// Get a random integer from random thread
pub fn get_random_int(low:u32, high:u32) -> u32 {
    let rng_float = rand::random::<f64>(); // get between [0, 1)
    
    let norm_seed = (rng_float / ((high-low) as f64) ) as u32 + low;
    return norm_seed
}


#[cfg(test)]
mod tests {

    use super::*;

    
    const EPS:f64 = 1e-9;
    fn float_compare(a:f64, b:f64) -> bool {
        return (a-b) < EPS
    }

    #[test]
    fn test_ratio_value() {
        let rat = PkmnRational{numer:5, demon:10};

        assert!(float_compare(rat.float(), 5.0/10.0));
        assert!(float_compare(
            PkmnRational::new(4,2).float(),
            4.0/2.0)
        );
    }

    #[test]
    fn test_ratio_math() {
        assert!(
            float_compare(
                (PkmnRational::new(10, 20) 
                + PkmnRational::new(5, 20)).float(),
                 15.0/20.0)
        );
        assert!(
            float_compare(
                (PkmnRational::new(10, 20) 
                - PkmnRational::new(5, 20)).float(),
                 5.0/20.0)
        );
        assert!(
            float_compare(
                (PkmnRational::new(1, 3) 
                + PkmnRational::new(3, 4)).float(),
                 13.0/12.0)
        );
        assert!(
            float_compare(
                (PkmnRational::new(2, 3) 
                - PkmnRational::new(1, 4)).float(),
                 11.0/12.0)
        );
        // TODO: Add assign test
    }

    #[test]
    fn test_ratio_reduction() {
        let rat = PkmnRational::new(240, 1080);
        let rat2 = PkmnRational::new(8, 80);

        let mut rat_sum = rat + rat2;
        assert_eq!(rat_sum.numer, 240*80 + 8 * 1080);
        assert_eq!(rat_sum.demon, 1080 * 80);

        
        // force reduction (note this will happen automatically in the future*)
        rat_sum.reduce();
        println!("{}", rat_sum);
        assert_eq!(rat_sum.numer, 29);
        assert_eq!(rat_sum.demon, 90);
    }

    #[test]
    fn get_random_int_in_range() {
        let range = 10..15;
        for _n in 0..10_000 {
            let rng = get_random_int(range.start, range.end);
            assert!(rng >= range.start && rng < range.end);
        }
    }

    #[test]
    fn test_rational_equality() {
        // same denominator
        assert!(PkmnRational { numer: 3, demon: 10 } == PkmnRational { numer: 3, demon: 10 });
        assert!(PkmnRational { numer: 2, demon: 10 } != PkmnRational { numer: 3, demon: 10 });

        // different denominator, equivalent value (1/2 == 2/4)
        assert!(PkmnRational { numer: 1, demon: 2 } == PkmnRational { numer: 2, demon: 4 });
        assert!(PkmnRational { numer: 1, demon: 3 } != PkmnRational { numer: 1, demon: 2 });
    }

    #[test]
    fn test_rational_multiply() {
        // scalar: 1/4 * 3 == 3/4
        let r = PkmnRational { numer: 1, demon: 4 } * 3;
        assert_eq!(r.numer, 3);
        assert_eq!(r.demon, 4);

        // rational * rational: 1/2 * 3/4 == 3/8
        let a = PkmnRational { numer: 1, demon: 2 };
        let b = PkmnRational { numer: 3, demon: 4 };
        let product = a * b;
        assert_eq!(product.numer, 3);
        assert_eq!(product.demon, 8);
    }

    #[test]
    fn test_rational_pow() {
        let r = PkmnRational::new(2, 3);

        // exponent 0 should return multiplicative identity.
        assert_eq!(r.pow(0), PkmnRational::ONE());

        // (2/3)^3 = 8/27
        assert_eq!(r.pow(3), PkmnRational::new(8, 27));

        // operator form delegates to the same multiplication-based power logic.
        assert_eq!(r ^ 3, PkmnRational::new(8, 27));
    }

    #[test]
    fn test_rational_product() {
        let vals = vec![
            PkmnRational::new(1, 2),
            PkmnRational::new(3, 4),
            PkmnRational::new(5, 6),
        ];

        // 1/2 * 3/4 * 5/6 = 15/48 = 5/16
        let by_ref: PkmnRational = vals.iter().product();
        assert_eq!(by_ref, PkmnRational::new(5, 16));

        let by_value: PkmnRational = vals.into_iter().product();
        assert_eq!(by_value, PkmnRational::new(5, 16));
    }

    #[test]
    fn test_gen_power_set_same_probability_1_to_3() {
        let p = PkmnRational::new(1, 4);
        let q = PkmnRational::new(3, 4);

        let cases: Vec<Vec<PkmnRational>> = vec![
            vec![q, p],
            vec![q * q, p * q, q * p, p * p],
            vec![
                q * q * q,
                p * q * q,
                q * p * q,
                p * p * q,
                q * q * p,
                p * q * p,
                q * p * p,
                p * p * p,
            ],
        ];

        for expected in cases {
            let probs = vec![p; expected.len().ilog2() as usize];
            let dist = gen_power_set(probs);

            assert_eq!(dist.len(), expected.len());
            for idx in 0..expected.len() {
                assert_eq!(dist[idx], expected[idx]);
            }

            let mut total = PkmnRational::ZERO();
            for prob in dist {
                total += prob;
            }
            assert_eq!(total, PkmnRational::ONE());
        }
    }

    #[test]
    fn test_gen_power_set_three_different_probabilities() {
        let probs = vec![
            PkmnRational::new(1, 5),
            PkmnRational::new(1, 3),
            PkmnRational::new(1, 4),
        ];

        let dist = gen_power_set(probs);
        let expected = vec![
            PkmnRational::new(2, 5),
            PkmnRational::new(1, 10),
            PkmnRational::new(1, 5),
            PkmnRational::new(1, 20),
            PkmnRational::new(2, 15),
            PkmnRational::new(1, 30),
            PkmnRational::new(1, 15),
            PkmnRational::new(1, 60),
        ];

        assert_eq!(dist.len(), expected.len());
        for idx in 0..expected.len() {
            assert_eq!(dist[idx], expected[idx]);
        }

        let mut total = PkmnRational::ZERO();
        for prob in dist {
            total += prob;
        }
        assert_eq!(total, PkmnRational::ONE());
    }

    #[test]
    /// Test the identity for the power_set
    fn test_get_power_set_one () {
        let result = gen_power_set(vec![PkmnRational::ONE()]);

        print!("{}", result[0]);
        assert_eq!(result[0], PkmnRational::ZERO());
        assert_eq!(result[1], PkmnRational::ONE());
    }
}
