use super::errors::AnalyzerError;
use std::collections::{HashMap, HashSet};
use wasmparser::Operator;

#[derive(Debug)]
struct LoopInfo {
    instruction_count: u32,
    has_break_condition: bool,
    has_control_flow: bool,
    modified_variables: HashSet<u32>,
    break_conditions: Vec<String>,
    condition_variables: HashSet<u32>, // Variables used in break conditions
}

#[derive(Debug)]
pub struct LoopAnalyzer {
    max_iterations: u32,
    max_loop_depth: u32,
    loop_stack: Vec<LoopInfo>,
    global_modified_vars: HashSet<u32>,
    variable_states: HashMap<u32, bool>, // Tracks if variables are used in conditions
}

impl LoopAnalyzer {
    pub fn new(max_iterations: u32, max_loop_depth: u32) -> Self {
        Self {
            max_iterations,
            max_loop_depth,
            loop_stack: Vec::new(),
            global_modified_vars: HashSet::new(),
            variable_states: HashMap::new(),
        }
    }

    fn check_loop_termination(&self, loop_info: &LoopInfo) -> bool {
        // If there's any control flow (function calls, etc.), assume it might terminate
        if loop_info.has_control_flow {
            return true;
        }

        // If there are no break conditions, it's definitely infinite
        if !loop_info.has_break_condition {
            return false;
        }

        // Check if any modified variables are used in break conditions
        let has_valid_condition = !loop_info.condition_variables.is_empty()
            && loop_info
                .modified_variables
                .intersection(&loop_info.condition_variables)
                .next()
                .is_some();

        // If we have break conditions but they don't depend on modified variables,
        // the loop might be infinite
        has_valid_condition
    }

    pub fn analyze_operators(&mut self, operators: &[Operator]) -> Result<(), AnalyzerError> {
        let mut current_depth = 0;
        let mut last_condition_vars = HashSet::new();

        for op in operators {
            match op {
                Operator::Loop { .. } => {
                    current_depth += 1;
                    if current_depth > self.max_loop_depth {
                        return Err(AnalyzerError::ExceededLoopDepth(self.max_loop_depth));
                    }

                    self.loop_stack.push(LoopInfo {
                        instruction_count: 0,
                        has_break_condition: false,
                        has_control_flow: false,
                        modified_variables: HashSet::new(),
                        break_conditions: Vec::new(),
                        condition_variables: HashSet::new(),
                    });
                }

                Operator::End => {
                    if !self.loop_stack.is_empty() {
                        let mut loop_info = self.loop_stack.pop().unwrap();

                        // Add variables from the last condition check
                        loop_info
                            .condition_variables
                            .extend(last_condition_vars.drain());

                        // Check for potential infinite loop
                        if !self.check_loop_termination(&loop_info) {
                            return Err(AnalyzerError::NoBreakCondition);
                        }
                    }
                    if current_depth > 0 {
                        current_depth -= 1;
                    }
                }

                Operator::BrIf { .. } | Operator::Br { .. } => {
                    if let Some(loop_info) = self.loop_stack.last_mut() {
                        loop_info.has_break_condition = true;
                        loop_info
                            .break_conditions
                            .push("Branch condition".to_string());
                    }
                }

                Operator::Call { .. } => {
                    if let Some(loop_info) = self.loop_stack.last_mut() {
                        loop_info.has_control_flow = true;
                    }
                }

                Operator::LocalGet { local_index } => {
                    // Track variables used in conditions
                    last_condition_vars.insert(*local_index);
                    if let Some(loop_info) = self.loop_stack.last_mut() {
                        loop_info.condition_variables.insert(*local_index);
                    }
                }

                Operator::LocalSet { local_index } | Operator::LocalTee { local_index } => {
                    if let Some(loop_info) = self.loop_stack.last_mut() {
                        loop_info.modified_variables.insert(*local_index);
                        self.variable_states.insert(*local_index, true);
                    } else {
                        self.global_modified_vars.insert(*local_index);
                    }
                }

                // Track comparison operators as they might be part of break conditions
                Operator::I32Eq { .. }
                | Operator::I32Ne { .. }
                | Operator::I32LtS { .. }
                | Operator::I32GtS { .. }
                | Operator::I32LeS { .. }
                | Operator::I32GeS { .. } => {
                    if let Some(loop_info) = self.loop_stack.last_mut() {
                        // The variables used before this comparison are potential condition variables
                        loop_info
                            .condition_variables
                            .extend(last_condition_vars.drain());
                    }
                }

                _ => {}
            }

            // Check for loop invariant violations
            if let Some(loop_info) = self.loop_stack.last_mut() {
                loop_info.instruction_count += 1;
                if loop_info.instruction_count > self.max_iterations {
                    return Err(AnalyzerError::ExceededLoopIterations(self.max_iterations));
                }
            }
        }

        Ok(())
    }
}
