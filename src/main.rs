//! NeuroScript CLI

use neuroscript::{parse, NeuronBody};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: neuroscript <file.ns>");
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    match parse(&source) {
        Ok(program) => {
            println!("Parsed {} imports and {} neurons:\n", program.uses.len(), program.neurons.len());

            for use_stmt in &program.uses {
                println!("  use {},{}", use_stmt.source, use_stmt.path.join("/"));
            }

            if !program.uses.is_empty() {
                println!();
            }

            for (name, neuron) in &program.neurons {
                let kind = match &neuron.body {
                    NeuronBody::Primitive(_) => "primitive",
                    NeuronBody::Graph(_) => "composite",
                };

                let inputs: Vec<_> = neuron.inputs.iter()
                    .map(|p| format!("{}: {}", p.name, p.shape))
                    .collect();

                let outputs: Vec<_> = neuron.outputs.iter()
                    .map(|p| format!("{}: {}", p.name, p.shape))
                    .collect();

                println!("  {} ({})", name, kind);
                println!("    in:  {}", inputs.join(", "));
                println!("    out: {}", outputs.join(", "));

                if let NeuronBody::Graph(conns) = &neuron.body {
                    println!("    connections: {}", conns.len());
                }

                println!();
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
