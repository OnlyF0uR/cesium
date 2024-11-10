const FIELD_PRIME: u64 = (1 << 61) - 1; // Mersenne prime

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldElement {
    pub value: u64,
}

impl FieldElement {
    pub fn new(value: u64) -> Self {
        Self {
            value: value % FIELD_PRIME,
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self::new(self.value.wrapping_add(other.value))
    }

    pub fn mul(&self, other: &Self) -> Self {
        Self::new((self.value as u128 * other.value as u128 % FIELD_PRIME as u128) as u64)
    }

    pub fn sub(&self, other: &Self) -> Self {
        if self.value >= other.value {
            Self::new(self.value - other.value)
        } else {
            Self::new(FIELD_PRIME - (other.value - self.value))
        }
    }

    pub fn square(&self) -> Self {
        self.mul(self)
    }

    pub fn exp(&self, exponent: u64) -> Self {
        let mut result = Self::new(1);
        let mut base = *self;

        let mut exp = exponent;
        while exp > 0 {
            if exp % 2 == 1 {
                result = result.mul(&base);
            }
            base = base.square();
            exp /= 2;
        }

        result
    }

    pub fn random(rng: &mut impl rand::RngCore) -> Self {
        Self::new(rng.next_u64())
    }
}
