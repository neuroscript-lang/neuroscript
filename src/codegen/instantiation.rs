//! Module instantiation logic for __init__ method generation
//!
//! This module handles generating module instantiations in the __init__ method,
//! including deduplication of calls and lazy instantiation for modules with
//! captured dimensions.

use super::utils::*;
use crate::interfaces::Kwarg;
use crate::interfaces::*;
use std::collections::{hash_map, HashMap};
use std::fmt::Write;

/// Generate module instantiations in __init__
pub(super) fn generate_module_instantiations(
    gen: &mut CodeGenerator,
    output: &mut String,
    context_bindings: &[Binding],
    connections: &[Connection],
) -> Result<(), CodegenError> {
    let mut instantiated_count = 0;

    // 1. Process unified context: bindings
    for binding in context_bindings {
        let module_name = binding.name.clone();
        let name = &binding.call_name;
        let args = &binding.args;
        let kwargs = &binding.kwargs;

        let is_primitive = if let Some(neuron) = gen.program.neurons.get(name.as_str()) {
            neuron.is_primitive()
        } else {
            true // Assume primitive if not in program
        };

        if is_primitive {
            gen.used_primitives.insert(name.clone());
        }

        match &binding.scope {
            Scope::Static => {
                // Static bindings are shared across all instances
                // We'll use a class-level variable for this
                // (Note: generator needs to Know class name, but currently it's not passed here)
                // For now, let's use a simpler approach: self.__class__.name
                let (args_str, kwargs_str) = extract_kwargs(args, kwargs);

                writeln!(
                    output,
                    "        if not hasattr(self.__class__, '{}'):",
                    module_name
                )
                .unwrap();
                writeln!(
                    output,
                    "            self.__class__.{} = {}({}{})",
                    module_name, name, args_str, kwargs_str
                )
                .unwrap();

                gen.var_names.insert(
                    module_name.clone(),
                    format!("self.__class__.{}", module_name),
                );
                instantiated_count += 1;
            }
            Scope::Instance { lazy: true } => {
                // Lazy instance binding
                writeln!(
                    output,
                    "        self._{} = None  # Lazy instantiation (@lazy)",
                    module_name
                )
                .unwrap();

                gen.lazy_bindings.insert(
                    module_name.clone(),
                    (name.clone(), args.clone(), kwargs.clone()),
                );
                gen.var_names
                    .insert(module_name.clone(), format!("self._{}", module_name));
                instantiated_count += 1;
            }
            Scope::Instance { lazy: false } => {
                // Eager instance binding
                let (args_str, kwargs_str) = extract_kwargs(args, kwargs);

                writeln!(
                    output,
                    "        self.{} = {}({}{})",
                    module_name, name, args_str, kwargs_str
                )
                .unwrap();

                gen.var_names
                    .insert(module_name.clone(), format!("self.{}", module_name));
                instantiated_count += 1;
            }
            Scope::Global => {
                // This shouldn't happen due to validation, but handle it anyway
                gen.var_names.insert(module_name.clone(), name.clone());
            }
        }
    }

    // 3. Collect and instantiate anonymous calls from connections
    let mut seen_calls: HashMap<String, (String, String, Vec<Value>, Vec<Kwarg>)> = HashMap::new();
    let mut all_endpoints = Vec::new();
    collect_calls_impl(connections, &mut all_endpoints);

    for endpoint in &all_endpoints {
        if let Endpoint::Call { .. } = endpoint {
            let key = endpoint_key_impl(endpoint);
            if let hash_map::Entry::Vacant(e) = seen_calls.entry(key.clone()) {
                let id = gen.next_node_id();
                let name = extract_call_name(endpoint);
                let module_name = format!("{}_{}", snake_case_impl(&name), id);
                let args = extract_call_args(endpoint);
                let kwargs = extract_call_kwargs(endpoint);

                gen.call_to_module.insert(key.clone(), module_name.clone());
                e.insert((name, module_name, args, kwargs));
            }
        }
    }

    let mut calls: Vec<_> = seen_calls.into_iter().collect();
    calls.sort_by(|a, b| a.1 .1.cmp(&b.1 .1));

    for (_key, (name, module_name, args, kwargs)) in &calls {
        let has_captured = args
            .iter()
            .any(|v| has_captured_dimensions_impl(v, &gen.current_neuron_params))
            || kwargs
                .iter()
                .any(|(_, v)| has_captured_dimensions_impl(v, &gen.current_neuron_params));

        if has_captured {
            writeln!(
                output,
                "        self._{} = None  # Lazy instantiation (captured)",
                module_name
            )
            .unwrap();
            instantiated_count += 1;
            continue;
        }

        if let Some(neuron) = gen.program.neurons.get(name.as_str()) {
            if neuron.is_primitive() {
                gen.used_primitives.insert(name.clone());
            }
        } else {
            gen.used_primitives.insert(name.clone());
        }

        let (args_str, kwargs_str) = extract_kwargs(args, kwargs);

        writeln!(
            output,
            "        self.{} = {}({}{})",
            module_name, name, args_str, kwargs_str
        )
        .unwrap();
        instantiated_count += 1;
    }

    if instantiated_count == 0 {
        writeln!(output, "        pass").unwrap();
    }

    Ok(())
}

fn extract_kwargs(args: &[Value], kwargs: &[(String, Value)]) -> (String, String) {
    let args_str = args
        .iter()
        .map(value_to_python_impl)
        .collect::<Vec<_>>()
        .join(", ");
    let kwargs_str = format_kwargs_impl(kwargs);
    (args_str, kwargs_str)
}

fn format_kwargs_impl(kwargs: &[(String, Value)]) -> String {
    if kwargs.is_empty() {
        String::new()
    } else {
        let kw: Vec<String> = kwargs
            .iter()
            .map(|(k, v)| format!("{}={}", k, value_to_python_impl(v)))
            .collect();
        format!(", {}", kw.join(", "))
    }
}

fn extract_call_name(endpoint: &Endpoint) -> String {
    match endpoint {
        Endpoint::Call { name, .. } => name.clone(),
        _ => String::new(),
    }
}

fn extract_call_args(endpoint: &Endpoint) -> Vec<Value> {
    match endpoint {
        Endpoint::Call { args, .. } => args.clone(),
        _ => vec![],
    }
}

fn extract_call_kwargs(endpoint: &Endpoint) -> Vec<Kwarg> {
    match endpoint {
        Endpoint::Call { kwargs, .. } => kwargs.clone(),
        _ => vec![],
    }
}

#[cfg(test)]
mod tests;
