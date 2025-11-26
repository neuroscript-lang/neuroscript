use crate::ir::*;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShapeError {
    #[error("Shape mismatch: expected {expected}, got {got}")]
    Mismatch { expected: Shape, got: Shape, context: String },

    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimMismatch { expected: Dim, got: Dim, context: String },

    #[error("Unknown dimension variable: {name}")]
    UnknownDim { name: String, context: String },

    #[error("Constraint violation: {message}")]
    ConstraintViolation { message: String, context: String },
    
    #[error("Inference failed for node {node}: {message}")]
    NodeInferenceFailed { node: String, message: String },

    #[error("Unknown node or port: {0}")]
    UnknownNode(String),
}

/// Tracks the state of dimension variables during inference
#[derive(Debug, Clone, Default)]
pub struct InferenceContext {
    /// Map from dimension name to its resolved value (if known)
    pub resolved_dims: HashMap<String, usize>,
    
    /// Map from dimension name to other equivalent dimension names
    pub equivalences: HashMap<String, String>,

    /// Map from named nodes to their output shapes
    /// e.g. "in" -> [Shape], "x" -> [Shape]
    pub node_outputs: HashMap<String, Vec<Shape>>,

    /// Map from anonymous call IDs to their output shapes
    /// e.g. Linear(512, 256) (id=1) -> [[*, 256]]
    pub call_outputs: HashMap<usize, Vec<Shape>>,
}

impl InferenceContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve_dim(&self, dim: &Dim) -> Option<usize> {
        match dim {
            Dim::Literal(n) => Some(*n as usize),
            Dim::Named(name) => self.resolved_dims.get(name).copied(),
            Dim::Expr(expr) => self.evaluate_expr(expr),
            _ => None,
        }
    }

    fn evaluate_expr(&self, expr: &DimExpr) -> Option<usize> {
        let left = self.resolve_dim(&expr.left)?;
        let right = self.resolve_dim(&expr.right)?;
        
        match expr.op {
            BinOp::Add => Some(left + right),
            BinOp::Sub => Some(left.checked_sub(right)?),
            BinOp::Mul => Some(left * right),
            BinOp::Div => Some(left / right),
            _ => None,
        }
    }
    
    pub fn unify(&mut self, d1: &Dim, d2: &Dim) -> Result<(), String> {
        match (d1, d2) {
            (Dim::Literal(v1), Dim::Literal(v2)) => {
                if v1 != v2 {
                    return Err(format!("Literal mismatch: {} != {}", v1, v2));
                }
            }
            (Dim::Named(n1), Dim::Named(n2)) => {
                if n1 != n2 {
                    let v1 = self.resolved_dims.get(n1);
                    let v2 = self.resolved_dims.get(n2);
                    
                    if let (Some(val1), Some(val2)) = (v1, v2) {
                        if val1 != val2 {
                            return Err(format!("Variable mismatch: {}={} != {}={}", n1, val1, n2, val2));
                        }
                    } else if let Some(val) = v1 {
                        self.resolved_dims.insert(n2.clone(), *val);
                    } else if let Some(val) = v2 {
                        self.resolved_dims.insert(n1.clone(), *val);
                    }
                }
            }
            (Dim::Named(n), Dim::Literal(v)) | (Dim::Literal(v), Dim::Named(n)) => {
                if let Some(current) = self.resolved_dims.get(n) {
                    if *current != *v as usize {
                        return Err(format!("Variable {} already bound to {}, cannot bind to {}", n, current, v));
                    }
                } else {
                    self.resolved_dims.insert(n.clone(), *v as usize);
                }
            }
            _ => {} 
        }
        Ok(())
    }
}

pub struct ShapeInferenceEngine {
}

impl ShapeInferenceEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn infer(&mut self, program: &Program) -> Result<(), Vec<ShapeError>> {
        let mut errors = Vec::new();
        
        for (name, neuron) in &program.neurons {
            if let Err(e) = self.infer_neuron(neuron, program) {
                errors.push(e);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn infer_neuron(&mut self, neuron: &NeuronDef, program: &Program) -> Result<(), ShapeError> {
        if neuron.is_primitive() {
            return Ok(());
        }

        let mut ctx = InferenceContext::new();
        
        // 1. Initialize context with known params
        for param in &neuron.params {
            if let Some(Value::Int(val)) = param.default {
                ctx.resolved_dims.insert(param.name.clone(), val as usize);
            }
        }

        // 2. Register input shapes
        // "in" node outputs = neuron inputs
        let input_shapes: Vec<Shape> = neuron.inputs.iter().map(|p| p.shape.clone()).collect();
        ctx.node_outputs.insert("in".to_string(), input_shapes.clone());
        
        // Also register individual input ports if they are named
        for (i, port) in neuron.inputs.iter().enumerate() {
            if port.name != "default" {
                ctx.node_outputs.insert(port.name.clone(), vec![port.shape.clone()]);
            }
        }

        // 3. Walk the graph
        if let NeuronBody::Graph(connections) = &neuron.body {
            for conn in connections {
                self.check_connection(conn, &mut ctx, program)?;
            }
        }

        Ok(())
    }

    fn check_connection(&self, conn: &Connection, ctx: &mut InferenceContext, program: &Program) -> Result<(), ShapeError> {
        // 1. Resolve source shapes
        let source_shapes = self.resolve_endpoint_shape(&conn.source, ctx, program)?;
        
        // 2. Handle destination
        match &conn.destination {
            Endpoint::Call { name, args: _, kwargs: _, id } => {
                // Instantiate the called neuron
                let called_neuron = program.neurons.get(name)
                    .ok_or_else(|| ShapeError::UnknownNode(name.clone()))?;

                // Unify source shapes with called neuron's inputs
                if source_shapes.len() != called_neuron.inputs.len() {
                     return Err(ShapeError::Mismatch { 
                         expected: Shape::new(vec![]), // TODO: better error
                         got: Shape::new(vec![]), 
                         context: format!("Arity mismatch calling {}: expected {}, got {}", name, called_neuron.inputs.len(), source_shapes.len())
                     });
                }

                for (src_shape, input_port) in source_shapes.iter().zip(called_neuron.inputs.iter()) {
                    self.unify_shapes(src_shape, &input_port.shape, ctx).map_err(|msg| ShapeError::ConstraintViolation {
                        message: msg,
                        context: format!("Input to {}", name),
                    })?;
                }

                // Compute output shapes of the called neuron
                // For now, just take the declared output shapes
                // TODO: Propagate specific dimension bindings into the called neuron context?
                // Yes, if we call Linear(d, 512) with input [*, 256], d becomes 256.
                // But `d` is a param of Linear? No, Linear(in, out).
                // If Linear is defined as: neuron Linear(in, out): ...
                // Then `in` and `out` are params.
                // The input port shape might use `in`.
                
                // For MVP: Just use the output shapes from definition, substituting params if possible.
                // But we need to substitute params based on `args`.
                // This is getting complex. For now, assume output shapes are static or depend on params which are literals.
                
                let output_shapes = called_neuron.outputs.iter().map(|p| p.shape.clone()).collect();
                ctx.call_outputs.insert(*id, output_shapes);
            }
            Endpoint::Ref(port_ref) => {
                // Assign source shapes to this node/port
                // If it's "out", we verify against neuron outputs
                if port_ref.node == "out" {
                    // Verify against neuron outputs
                    // TODO: Handle named ports
                     // For now assume single output or matching order
                     // This is tricky without full symbol table logic
                } else {
                    // Intermediate node
                    ctx.node_outputs.insert(port_ref.node.clone(), source_shapes);
                }
            }
            Endpoint::Tuple(refs) => {
                if source_shapes.len() != refs.len() {
                    return Err(ShapeError::Mismatch {
                        expected: Shape::new(vec![]),
                        got: Shape::new(vec![]),
                        context: "Tuple unpacking arity mismatch".to_string()
                    });
                }
                for (shape, port_ref) in source_shapes.iter().zip(refs.iter()) {
                    ctx.node_outputs.insert(port_ref.node.clone(), vec![shape.clone()]);
                }
            }
            Endpoint::Match(_) => {
                // TODO
            }
        }

        Ok(())
    }

    fn resolve_endpoint_shape(&self, endpoint: &Endpoint, ctx: &InferenceContext, _program: &Program) -> Result<Vec<Shape>, ShapeError> {
        match endpoint {
            Endpoint::Ref(port_ref) => {
                if let Some(shapes) = ctx.node_outputs.get(&port_ref.node) {
                    // TODO: Handle port_ref.port selection
                    Ok(shapes.clone())
                } else {
                    Err(ShapeError::UnknownNode(port_ref.node.clone()))
                }
            }
            Endpoint::Call { id, .. } => {
                if let Some(shapes) = ctx.call_outputs.get(id) {
                    Ok(shapes.clone())
                } else {
                    // This happens if we try to read from a Call that hasn't been processed as a destination yet?
                    // But connections are ordered?
                    // Or if it's a source-only call (generator)?
                    // If it's a source, we need to instantiate it and get outputs.
                    // But we don't have inputs to unify.
                    // TODO: Handle source calls
                    Ok(vec![]) 
                }
            }
            Endpoint::Tuple(refs) => {
                let mut shapes = Vec::new();
                for r in refs {
                    let s = self.resolve_endpoint_shape(&Endpoint::Ref(r.clone()), ctx, _program)?;
                    shapes.extend(s);
                }
                Ok(shapes)
            }
            Endpoint::Match(_) => Ok(vec![]), // TODO
        }
    }

    fn unify_shapes(&self, s1: &Shape, s2: &Shape, ctx: &mut InferenceContext) -> Result<(), String> {
        if s1.dims.len() != s2.dims.len() {
            // Check for variadic?
            return Err(format!("Rank mismatch: {} vs {}", s1, s2));
        }

        for (d1, d2) in s1.dims.iter().zip(s2.dims.iter()) {
            ctx.unify(d1, d2)?;
        }
        Ok(())
    }
}
