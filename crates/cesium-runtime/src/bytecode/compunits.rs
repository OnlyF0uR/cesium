use once_cell::sync::Lazy;
use std::collections::HashMap;
use wasmparser::Operator;

/// Represents the cost metrics for different types of operations
#[derive(Debug, Clone, Copy)]
struct ComputationCosts {
    base_cost: u64,
    memory_cost: u64,
    compute_cost: u64,
}

impl ComputationCosts {
    #[inline(always)]
    fn total(&self) -> u64 {
        self.base_cost + self.memory_cost + self.compute_cost
    }
}

/// Static cost table initialized once using Lazy
static COST_TABLE: Lazy<HashMap<&'static str, ComputationCosts>> = Lazy::new(|| {
    let mut map = HashMap::with_capacity(40); // Pre-allocate capacity

    // Control flow operations
    map.extend([
        (
            "block",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 0,
                compute_cost: 0,
            },
        ),
        (
            "loop",
            ComputationCosts {
                base_cost: 3,
                memory_cost: 0,
                compute_cost: 0,
            },
        ),
        (
            "br",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 0,
                compute_cost: 0,
            },
        ),
        (
            "br_if",
            ComputationCosts {
                base_cost: 3,
                memory_cost: 0,
                compute_cost: 1,
            },
        ),
        (
            "end",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 0,
            },
        ),
        (
            "return",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 0,
                compute_cost: 0,
            },
        ),
    ]);

    // Memory operations
    map.extend([
        (
            "i32.load8_u",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 3,
                compute_cost: 0,
            },
        ),
        (
            "i32.load",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 3,
                compute_cost: 0,
            },
        ),
        (
            "i32.store",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 3,
                compute_cost: 0,
            },
        ),
    ]);

    // Local variable operations
    map.extend([
        (
            "local.get",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 0,
            },
        ),
        (
            "local.set",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 0,
            },
        ),
        (
            "local.tee",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        ),
    ]);

    // Add other sections similarly...
    map.extend([
        (
            "i32.add",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        ),
        (
            "i32.sub",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        ),
        (
            "i32.mul",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 2,
            },
        ),
        (
            "i32.div_u",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 3,
            },
        ),
        // ... other arithmetic operations
    ]);

    map
});

/// Determine the cost of a single WebAssembly operator
pub fn calculate_operator_cost(op: &Operator) -> u64 {
    let op_name = match op {
        Operator::Block { .. } => "block",
        Operator::Loop { .. } => "loop",
        Operator::Br { .. } => "br",
        Operator::BrIf { .. } => "br_if",
        Operator::End => "end",
        Operator::Return => "return",
        Operator::LocalGet { .. } => "local.get",
        Operator::LocalSet { .. } => "local.set",
        Operator::LocalTee { .. } => "local.tee",
        Operator::I32Load8U { .. } => "i32.load8_u",
        Operator::I32Add => "i32.add",
        Operator::I32Sub => "i32.sub",
        Operator::I32Mul => "i32.mul",
        Operator::I32DivU { .. } => "i32.div_u",
        Operator::I32And => "i32.and",
        Operator::I32Const { .. } => "i32.const",
        Operator::I64Const { .. } => "i64.const",
        _ => return 0,
    };

    COST_TABLE
        .get(op_name)
        .map(|costs| costs.total())
        .unwrap_or(0)
}

/// Calculate operation costs for a sequence of WebAssembly instructions
pub fn calculate_computational_costs(operators: &[Operator]) -> (u32, u64) {
    let mut total_cost = 0u64;
    let mut instr_count = 0u32;

    for op in operators {
        let cost = calculate_operator_cost(op);
        instr_count += 1;
        total_cost += cost;
    }

    (instr_count, total_cost)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmparser::BlockType;

    #[test]
    fn test_calculate_computational_costs() {
        let operators = vec![
            Operator::Block {
                blockty: BlockType::Empty,
            },
            Operator::I32Const { value: 42 },
            Operator::I32Const { value: 100 },
            Operator::I32Add,
            Operator::End,
        ];

        let (instr_count, compunits) = calculate_computational_costs(&operators);
        assert_eq!(instr_count, 5);
        assert_eq!(compunits, 5);
    }

    #[test]
    fn test_single_operator_cost() {
        let op = Operator::I32Add;
        let cost = calculate_operator_cost(&op);
        assert_eq!(cost, 2); // 1 base + 1 compute
    }
}
