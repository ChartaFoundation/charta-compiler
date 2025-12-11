use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::error::{Result, CompileError};
use crate::{parse, resolve_names, emit_ir};
use charta_core::ir::validation::validate_ir;
use charta_vm::VM;
use charta_vm::ir::load_ir;
use std::collections::HashMap;
use std::fs;

#[derive(Parser)]
#[command(name = "charta")]
#[command(about = "Charta compiler and runtime", version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compile Charta source to IR
    Compile {
        /// Input Charta source file
        #[arg(short, long)]
        input: PathBuf,
        /// Output IR file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Run IR program on VM
    Run {
        /// Input IR file
        #[arg(short, long)]
        input: PathBuf,
        /// Input values as JSON (optional)
        #[arg(long)]
        inputs: Option<String>,
    },
    /// Validate Charta source file
    Validate {
        /// Input Charta source file
        #[arg(short, long)]
        input: PathBuf,
    },
    /// Inspect IR file
    Inspect {
        /// Input IR file
        #[arg(short, long)]
        input: PathBuf,
    },
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Compile { input, output } => {
            compile_command(&input, output.as_ref())?;
        }
        Commands::Run { input, inputs } => {
            run_command(&input, inputs.as_deref())?;
        }
        Commands::Validate { input } => {
            validate_command(&input)?;
        }
        Commands::Inspect { input } => {
            inspect_command(&input)?;
        }
    }
    
    Ok(())
}

fn compile_command(input: &PathBuf, output: Option<&PathBuf>) -> Result<()> {
    let source = fs::read_to_string(input)
        .map_err(CompileError::Io)?;
    
    // Parse
    let mut module = parse(&source)?;
    
    // Resolve names
    resolve_names(&mut module)?;
    
    // Emit IR
    let ir_json = emit_ir(&module)?;
    
    // Write output
    let output_path = output.map(|p| p.clone())
        .unwrap_or_else(|| {
            input.with_extension("ir.json")
        });
    
    fs::write(&output_path, ir_json)
        .map_err(CompileError::Io)?;
    
    println!("Compiled {} to {}", input.display(), output_path.display());
    Ok(())
}

fn run_command(input: &PathBuf, inputs_json: Option<&str>) -> Result<()> {
    let ir_content = fs::read_to_string(input)
        .map_err(CompileError::Io)?;
    
    // Load IR
    let ir = load_ir(&ir_content)
        .map_err(|e| CompileError::Emission(format!("IR load error: {:?}", e)))?;
    
    // Create VM and load program
    let mut vm = VM::new();
    vm.load_program(ir)
        .map_err(|e| CompileError::Emission(format!("VM load error: {:?}", e)))?;
    
    // Parse inputs
    let mut inputs = HashMap::new();
    if let Some(inputs_str) = inputs_json {
        let parsed: HashMap<String, bool> = serde_json::from_str(inputs_str)
            .map_err(|e| CompileError::Emission(format!("Invalid inputs JSON: {}", e)))?;
        inputs = parsed;
    }
    
    // Execute cycle
    let outputs = vm.step(inputs)
        .map_err(|e| CompileError::Emission(format!("VM execution error: {:?}", e)))?;
    
    // Display results
    println!("Coil states:");
    for (name, value) in &outputs {
        println!("  {}: {}", name, value);
    }
    
    Ok(())
}

fn validate_command(input: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(input)
        .map_err(CompileError::Io)?;
    
    // Parse
    let mut module = parse(&source)?;
    
    // Resolve names
    resolve_names(&mut module)?;
    
    // Emit IR
    let ir_json = emit_ir(&module)?;
    
    // Validate IR against schema
    let schema_path = "../../spec/ir-schema.json";
    validate_ir(&ir_json, schema_path)
        .map_err(|e| CompileError::Emission(format!("IR validation error: {:?}", e)))?;
    
    println!("Validation successful: {}", input.display());
    Ok(())
}

fn inspect_command(input: &PathBuf) -> Result<()> {
    let ir_content = fs::read_to_string(input)
        .map_err(CompileError::Io)?;
    
    // Parse IR
    let ir: charta_core::ir::schema::IR = serde_json::from_str(&ir_content)
        .map_err(|e| CompileError::Emission(format!("Invalid IR JSON: {}", e)))?;
    
    // Display IR structure
    println!("Module: {}", ir.module.name);
    if let Some(context) = &ir.module.context {
        println!("Context: {}", context);
    }
    
    if let Some(signals) = &ir.module.signals {
        println!("\nSignals ({}):", signals.len());
        for signal in signals {
            println!("  - {}", signal.name);
        }
    }
    
    if let Some(coils) = &ir.module.coils {
        println!("\nCoils ({}):", coils.len());
        for coil in coils {
            println!("  - {} (latching: {}, critical: {})",
                coil.name,
                coil.latching.unwrap_or(false),
                coil.critical.unwrap_or(false)
            );
        }
    }
    
    if let Some(rungs) = &ir.module.rungs {
        println!("\nRungs ({}):", rungs.len());
        for rung in rungs {
            println!("  - {}", rung.name);
        }
    }
    
    Ok(())
}
