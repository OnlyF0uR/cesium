use std::collections::HashMap;

use wasmparser::Operator;

/// Represents the cost metrics for different types of operations
#[derive(Debug, Clone)]
struct ComputationCosts {
    base_cost: u64,
    memory_cost: u64,
    compute_cost: u64,
}

impl ComputationCosts {
    fn total(&self) -> u64 {
        self.base_cost + self.memory_cost + self.compute_cost
    }
}

/// Calculate operation costs for a sequence of WebAssembly instructions
pub fn calculate_computational_costs(operators: &[Operator]) -> (u32, u64) {
    // Define base costs for different operation types
    let cost_table: HashMap<&str, ComputationCosts> = {
        let mut map = HashMap::new();

        // Control flow operations
        map.insert(
            "block",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 0,
                compute_cost: 0,
            },
        );
        map.insert(
            "loop",
            ComputationCosts {
                base_cost: 3,
                memory_cost: 0,
                compute_cost: 0,
            },
        );
        map.insert(
            "br",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 0,
                compute_cost: 0,
            },
        );
        map.insert(
            "br_if",
            ComputationCosts {
                base_cost: 3,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "end",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 0,
            },
        );
        map.insert(
            "return",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 0,
                compute_cost: 0,
            },
        );

        // Memory operations
        map.insert(
            "i32.load8_u",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 3,
                compute_cost: 0,
            },
        );
        map.insert(
            "i32.load",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 3,
                compute_cost: 0,
            },
        );
        map.insert(
            "i32.store",
            ComputationCosts {
                base_cost: 2,
                memory_cost: 3,
                compute_cost: 0,
            },
        );

        // Local variable operations
        map.insert(
            "local.get",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 0,
            },
        );
        map.insert(
            "local.set",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 0,
            },
        );
        map.insert(
            "local.tee",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );

        // Arithmetic operations
        map.insert(
            "i32.add",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i32.sub",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i32.mul",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 2,
            },
        );
        map.insert(
            "i32.div_u",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 3,
            },
        );

        // Bitwise operations
        map.insert(
            "i32.and",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i32.or",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i32.xor",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i32.shl",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 2,
            },
        );
        map.insert(
            "i32.shr_u",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 2,
            },
        );
        map.insert(
            "i64.shr_u",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 2,
            },
        );

        // Comparison operations
        map.insert(
            "i32.eq",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i32.ne",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i32.lt_u",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i32.gt_u",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i32.le_u",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i32.ge_u",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );

        // Conversion operations
        map.insert(
            "i32.wrap_i64",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );
        map.insert(
            "i64.extend_i32_u",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 1,
            },
        );

        // Constant operations
        map.insert(
            "i32.const",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 0,
            },
        );
        map.insert(
            "i64.const",
            ComputationCosts {
                base_cost: 1,
                memory_cost: 0,
                compute_cost: 0,
            },
        );

        map
    };

    let mut total_cost = 0u64;
    let mut instr_count = 0u32;

    for op in operators {
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
            Operator::I32And => "i32.and",
            Operator::I32Const { .. } => "i32.const",
            Operator::I64Const { .. } => "i64.const",
            Operator::I32WrapI64 => "i32.wrap_i64",
            Operator::I64ShrU => "i64.shr_u",
            Operator::I32GeU => "i32.ge_u",
            Operator::I32Ne => "i32.ne",
            Operator::I32Eqz => "i32.eq",
            _ => "other",
        };

        if let Some(costs) = cost_table.get(op_name) {
            instr_count += 1;
            total_cost += costs.total();
        }
    }

    (instr_count, total_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_computational_costs() {
        let operators = vec![
            Operator::Block {
                blockty: wasmparser::BlockType::Empty,
            },
            Operator::I32Const { value: 42 },
            Operator::I32Const { value: 100 },
            Operator::I32Add,
            Operator::End,
        ];

        let (instr_count, compunits) = calculate_computational_costs(&operators);
        assert_eq!(instr_count, 5);
        assert_eq!(compunits, 7);
    }
}
