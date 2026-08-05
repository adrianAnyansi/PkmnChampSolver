use rand::rngs::ThreadRng;

/// Basic functions

/// Multiplies an integer by a float and floors the result to the nearest integer.
/// Used to calculate many stat modifications and damage rolls
pub fn mult_and_round(a: i32, b: f64) -> i32 {
    (a as f64 * b).round() as i32
}

pub fn div_and_floor(a: i32, b: i32) -> i32 {
    (a as f64 / b as f64).round() as i32
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
            rhs.numer * self.demon as i32 - self.numer * rhs.demon as i32, 
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

impl std::ops::BitXor<u32> for PkmnRational {
    type Output = Self;

    fn bitxor(self, rhs: u32) -> Self {
        self.pow(rhs)
    }
}



// TODO: Use this rng thread 
pub fn getRNGThread() -> ThreadRng {
    return rand::rng();
}

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
}
