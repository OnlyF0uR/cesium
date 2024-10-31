use wasmparser::{ExternalKind, FuncType, Operator, Parser, Payload, RecGroup, TypeRef};

use super::{compunits::calculate_computational_costs, errors::AnalyzerError, loop_analyzer};

// const array of allowed imports
const ALLOWED_IMPORTS: [&str; 10] = [
    "h_define_state",
    "h_get_state",
    "h_change_state",
    "h_commit_state",
    "h_initialize_data_account",
    "h_initialize_independent_data_account",
    "h_update_data_account",
    "h_commit_account_data",
    "h_commit_all",
    "h_gen_id",
];

#[derive(Debug)]
pub struct Function<'a> {
    pub name: String,
    pub import: bool,
    pub types: Vec<FuncType>,
    pub operators: Option<Vec<Operator<'a>>>,
}

#[derive(Debug)]
pub struct ImportFunction {
    pub name: String,
    pub type_ref: TypeRef,
}

#[derive(Debug)]
pub struct ExportFunction {
    pub name: String,
    pub index: u32,
}

#[derive(Debug)]
pub struct CodeSection<'a> {
    pub operators: Vec<Operator<'a>>,
    pub comp_costs: u64,
}

pub struct Analyzer {
    compunit_limit_per_func: u64,
    instr_limit_per_func: u32,
}

impl Analyzer {
    pub fn new(compunit_limit_per_func: u64, instr_limit_per_func: u32) -> Self {
        Self {
            compunit_limit_per_func,
            instr_limit_per_func,
        }
    }

    pub fn analyze<'a>(&mut self, bytecode: &'a [u8]) -> Result<Vec<Function<'a>>, AnalyzerError> {
        let mut import_functions: Vec<ImportFunction> = Vec::new();
        let mut types: Vec<RecGroup> = Vec::new();
        let mut function_type_indeces: Vec<u32> = Vec::new();
        let mut export_functions: Vec<ExportFunction> = Vec::new();
        let mut code_sections: Vec<CodeSection> = Vec::new();

        let mut la = loop_analyzer::LoopAnalyzer::new(1000, 5);

        for payload in Parser::new(0).parse_all(bytecode) {
            match payload.map_err(|e| AnalyzerError::ParserError(e.to_string()))? {
                Payload::Version { .. } => {
                    println!("====== Module");
                }
                Payload::ImportSection(s) => {
                    for import in s {
                        let import =
                            import.map_err(|e| AnalyzerError::ParserError(e.to_string()))?;
                        println!("  Import {}::{}", import.module, import.name);

                        if import.module != "env" || !ALLOWED_IMPORTS.contains(&import.name) {
                            return Err(AnalyzerError::DisallowedImport(
                                import.module.to_string(),
                                import.name.to_string(),
                            ));
                        }

                        import_functions.push(ImportFunction {
                            name: import.name.to_string(),
                            type_ref: import.ty,
                        });
                    }
                }
                Payload::TypeSection(t) => {
                    for ty in t {
                        let ty = ty.map_err(|e| AnalyzerError::ParserError(e.to_string()))?;
                        types.push(ty);
                    }
                }
                Payload::CodeSectionEntry(code) => {
                    let reader = code.get_operators_reader().map_err(|e| {
                        AnalyzerError::ParserError(format!("Error reading operators: {:?}", e))
                    })?;

                    let mut operators = Vec::new();
                    for item in reader {
                        let item = item.map_err(|e| AnalyzerError::ParserError(e.to_string()))?;
                        operators.push(item);
                    }

                    // Analyze for infinite loops
                    la.analyze_operators(&operators)?;

                    // IDEA: Propagate unit costs to the level of the calculate_computational_costs function
                    let (instr_count, comp_costs) = calculate_computational_costs(&operators);
                    if instr_count > self.instr_limit_per_func {
                        return Err(AnalyzerError::ExceededInstructionLimit(
                            self.instr_limit_per_func,
                        ));
                    }
                    if comp_costs > self.compunit_limit_per_func {
                        return Err(AnalyzerError::ExceededCompUnitLimit(
                            self.compunit_limit_per_func,
                        ));
                    }

                    code_sections.push(CodeSection {
                        operators,
                        comp_costs,
                    });
                }
                Payload::FunctionSection(function) => {
                    for f in function {
                        let f = f.map_err(|e| AnalyzerError::ParserError(e.to_string()))?;
                        function_type_indeces.push(f);
                    }
                }
                Payload::ExportSection(s) => {
                    for export in s {
                        let export =
                            export.map_err(|e| AnalyzerError::ParserError(e.to_string()))?;

                        if export.kind == ExternalKind::Func {
                            export_functions.push(ExportFunction {
                                name: export.name.to_string(),
                                index: export.index,
                            });
                        }
                    }
                }
                _other => {
                    // println!("{:?}", _other);
                }
            }
        }

        let mut true_functions: Vec<Function> = Vec::new();
        // First we will take care of the import functions
        for (_, import) in import_functions.iter().enumerate() {
            match import.type_ref {
                TypeRef::Func(type_index) => {
                    let rc = types[type_index as usize].clone();

                    let mut func_types: Vec<FuncType> = Vec::new();
                    rc.types().for_each(|t| {
                        func_types.push(t.unwrap_func().clone());
                    });

                    true_functions.push(Function {
                        name: import.name.to_string(),
                        import: true,
                        types: func_types,
                        operators: None,
                    });
                }
                _ => {
                    panic!("Invalid import type");
                }
            }
        }

        // Now we will take care of the export functions
        if function_type_indeces.len() != export_functions.len()
            || function_type_indeces.len() != code_sections.len()
        {
            panic!("Function type indeces and export functions do not match");
        }

        // loop trough the external functions
        for (_, export) in export_functions.iter().enumerate() {
            let type_index = function_type_indeces[export.index as usize - 1];
            let rc = types[type_index as usize].clone();

            let mut func_types: Vec<FuncType> = Vec::new();
            rc.types().for_each(|t| {
                func_types.push(t.unwrap_func().clone());
            });

            let cs = &code_sections[export.index as usize - 1];
            true_functions.push(Function {
                name: export.name.to_string(),
                import: false,
                types: func_types,
                operators: Some(cs.operators.clone()),
            });
        }

        // println!("True functions: {:?}", true_functions);
        pretty_print_functions(&true_functions);

        Ok(true_functions)
    }
}

fn pretty_print_functions(functions: &Vec<Function<'_>>) {
    for func in functions {
        println!("\n========================================================");
        println!("Function: {} (import: {})", func.name, func.import);
        println!("Types: {:?}", func.types);
        println!("Operators: {:?}", func.operators);
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, process::Command};

    use super::*;

    fn compile(package: &str) {
        Command::new("cargo")
            .args([
                "build",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
                "--package",
                package,
            ])
            .status()
            .expect("Failed to compile contract");
    }

    #[test]
    fn test_analyzer() {
        compile("nomisma");
        let mut file =
            File::open("../../target/wasm32-unknown-unknown/release/nomisma.wasm").unwrap();

        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();

        let mut analyzer = Analyzer::new(2400, 1800);

        let result = analyzer.analyze(&wasm_bytes);
        assert!(result.is_ok());

        let result = result.unwrap();
        pretty_print_functions(&result);
    }
}
