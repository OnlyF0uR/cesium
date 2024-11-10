use super::fields::FieldElement;

// We are currently not using this, but we could
// in the future use polynomial commitments to
// in the bulletproofs implementation.

#[derive(Clone, Debug)]
pub struct Polynomial {
    pub coefficients: Vec<FieldElement>,
}

impl Polynomial {
    pub fn new(coefficients: Vec<FieldElement>) -> Self {
        Self { coefficients }
    }

    pub fn zero() -> Self {
        Self {
            coefficients: vec![FieldElement::new(0)],
        }
    }

    pub fn evaluate(&self, point: &FieldElement) -> FieldElement {
        let mut result = FieldElement::new(0);
        let mut power = FieldElement::new(1);

        for coeff in &self.coefficients {
            result = result.add(&power.mul(coeff));
            power = power.mul(point);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polynomial() {
        let coefficients = vec![
            FieldElement::new(1),
            FieldElement::new(2),
            FieldElement::new(3),
        ];
        let poly = Polynomial::new(coefficients.clone());

        let point = FieldElement::new(2);
        let result = poly.evaluate(&point);

        let expected = FieldElement::new(1)
            .add(&FieldElement::new(2).mul(&point))
            .add(&FieldElement::new(3).mul(&point.square()));

        assert_eq!(result, expected);
    }
}
