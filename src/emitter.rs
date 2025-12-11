use crate::ast;
use crate::error::{CompileError, Result};
use charta_core::ir::schema::{
    IR, Module as IRModule, Intent, Constraints, DataPrivacy, Quality, Cost,
    SignalDecl, CoilDecl, RungDecl, GuardExpr, Action, Expr,
    BlockDecl, PortDecl, NetworkDecl, Wire, Output,
};
use serde_json;

/// Emit IR from AST
pub fn emit_ir(module: &ast::Module) -> Result<String> {
    let ir = IR {
        version: "0.1.0".to_string(),
        module: emit_module(module)?,
    };
    
    serde_json::to_string_pretty(&ir)
        .map_err(|e| CompileError::Emission(format!("JSON serialization error: {}", e)))
}

fn emit_module(module: &ast::Module) -> Result<IRModule> {
    Ok(IRModule {
        name: module.name.clone(),
        context: module.context.clone(),
        intent: module.intent.as_ref().map(emit_intent),
        constraints: module.constraints.as_ref().map(emit_constraints),
        signals: Some(module.signals.iter().map(emit_signal).collect()),
        coils: Some(module.coils.iter().map(emit_coil).collect()),
        rungs: Some(module.rungs.iter().map(|r| emit_rung(r)).collect::<Result<Vec<_>>>()?),
        blocks: Some(module.blocks.iter().map(emit_block).collect()),
        networks: Some(module.networks.iter().map(emit_network).collect()),
    })
}

fn emit_intent(intent: &ast::Intent) -> Intent {
    Intent {
        goal: intent.goal.clone(),
    }
}

fn emit_constraints(constraints: &ast::Constraints) -> Constraints {
    Constraints {
        data_privacy: constraints.data_privacy.as_ref().map(|dp| DataPrivacy {
            jurisdiction: dp.jurisdiction.clone(),
            pii_handling: dp.pii_handling.clone(),
        }),
        quality: constraints.quality.as_ref().map(|q| Quality {
            min_precision: q.min_precision,
            min_recall: q.min_recall,
        }),
        cost: constraints.cost.as_ref().map(|c| Cost {
            max_cost_per_submission: c.max_cost_per_submission.clone(),
        }),
    }
}

fn emit_signal(signal: &ast::SignalDecl) -> SignalDecl {
    SignalDecl {
        name: signal.name.clone(),
        parameters: if signal.parameters.is_empty() {
            None
        } else {
            Some(signal.parameters.clone())
        },
        type_: signal.type_.clone(),
    }
}

fn emit_coil(coil: &ast::CoilDecl) -> CoilDecl {
    CoilDecl {
        name: coil.name.clone(),
        parameters: if coil.parameters.is_empty() {
            None
        } else {
            Some(coil.parameters.clone())
        },
        latching: coil.latching,
        critical: coil.critical,
    }
}

fn emit_rung(rung: &ast::RungDecl) -> Result<RungDecl> {
    Ok(RungDecl {
        name: rung.name.clone(),
        guard: emit_guard(&rung.guard)?,
        actions: rung.actions.iter().map(emit_action).collect(),
    })
}

fn emit_guard(guard: &ast::GuardExpr) -> Result<GuardExpr> {
    match guard {
        ast::GuardExpr::Contact { name, contact_type, arguments } => {
            Ok(GuardExpr::Contact {
                name: name.clone(),
                contact_type: match contact_type {
                    ast::ContactType::NO => "NO".to_string(),
                    ast::ContactType::NC => "NC".to_string(),
                },
                arguments: Some(arguments.iter().map(emit_expr).collect()),
            })
        }
        ast::GuardExpr::And { left, right } => {
            Ok(GuardExpr::And {
                left: Box::new(emit_guard(left)?),
                right: Box::new(emit_guard(right)?),
            })
        }
        ast::GuardExpr::Or { left, right } => {
            Ok(GuardExpr::Or {
                left: Box::new(emit_guard(left)?),
                right: Box::new(emit_guard(right)?),
            })
        }
        ast::GuardExpr::Not { expr } => {
            Ok(GuardExpr::Not {
                expr: Box::new(emit_guard(expr)?),
            })
        }
    }
}

fn emit_expr(expr: &ast::Expr) -> Expr {
    match expr {
        ast::Expr::String(s) => Expr::String(s.clone()),
        ast::Expr::Number(n) => Expr::Number(*n),
        ast::Expr::Boolean(b) => Expr::Boolean(*b),
        ast::Expr::Identifier(id) => Expr::Identifier(id.clone()),
    }
}

fn emit_action(action: &ast::Action) -> Action {
    Action {
        action_type: match action.action_type {
            ast::ActionType::Energise => "energise".to_string(),
            ast::ActionType::DeEnergise => "de_energise".to_string(),
            ast::ActionType::Escalate => "escalate".to_string(),
            ast::ActionType::Require => "require".to_string(),
        },
        coil: action.coil.clone(),
        arguments: if action.arguments.is_empty() {
            None
        } else {
            Some(action.arguments.iter().map(emit_expr).collect())
        },
    }
}

fn emit_block(block: &ast::BlockDecl) -> BlockDecl {
    BlockDecl {
        name: block.name.clone(),
        inputs: Some(block.inputs.iter().map(|p| PortDecl {
            name: p.name.clone(),
            type_: p.type_.clone(),
        }).collect()),
        outputs: Some(block.outputs.iter().map(|p| PortDecl {
            name: p.name.clone(),
            type_: p.type_.clone(),
        }).collect()),
        effect: block.effect.clone(),
    }
}

fn emit_network(network: &ast::NetworkDecl) -> NetworkDecl {
    NetworkDecl {
        name: network.name.clone(),
        wires: Some(network.wires.iter().map(|w| Wire {
            source: w.source.clone(),
            target: w.target.clone(),
        }).collect()),
        outputs: Some(network.outputs.iter().map(|o| Output {
            name: o.name.clone(),
            source: o.source.clone(),
        }).collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast;

    #[test]
    fn test_emit_basic_ir() {
        let module = ast::Module {
            name: "test".to_string(),
            context: None,
            intent: None,
            constraints: None,
            signals: vec![ast::SignalDecl {
                name: "input".to_string(),
                parameters: Vec::new(),
                type_: None,
            }],
            coils: vec![ast::CoilDecl {
                name: "output".to_string(),
                parameters: Vec::new(),
                latching: None,
                critical: None,
            }],
            rungs: vec![ast::RungDecl {
                name: "r1".to_string(),
                guard: ast::GuardExpr::Contact {
                    name: "input".to_string(),
                    contact_type: ast::ContactType::NO,
                    arguments: Vec::new(),
                },
                actions: vec![ast::Action {
                    action_type: ast::ActionType::Energise,
                    coil: "output".to_string(),
                    arguments: Vec::new(),
                }],
            }],
            blocks: Vec::new(),
            networks: Vec::new(),
        };
        
        let ir_json = emit_ir(&module).unwrap();
        assert!(ir_json.contains("test"));
        assert!(ir_json.contains("input"));
        assert!(ir_json.contains("output"));
    }
}
