use logos::Logos;
use crate::ast::*;
use crate::error::{CompileError, Result};

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//[^\n]*")]
pub enum Token {
    // Keywords
    #[token("module")]
    Module,
    #[token("signal")]
    Signal,
    #[token("coil")]
    Coil,
    #[token("rung")]
    Rung,
    #[token("block")]
    Block,
    #[token("network")]
    Network,
    #[token("when")]
    When,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("energise")]
    Energise,
    #[token("de_energise")]
    DeEnergise,
    #[token("escalate")]
    Escalate,
    #[token("require")]
    Require,
    #[token("NO")]
    NO,
    #[token("NC")]
    NC,
    #[token("AND")]
    And,
    #[token("OR")]
    Or,
    #[token("NOT")]
    Not,
    #[token("inputs")]
    Inputs,
    #[token("outputs")]
    Outputs,
    #[token("internals")]
    Internals,
    #[token("implementation")]
    Implementation,
    #[token("effect")]
    Effect,
    #[token("context")]
    Context,
    #[token("intent")]
    Intent,
    #[token("constraints")]
    Constraints,
    #[token("wires")]
    Wires,
    
    // Literals
    #[regex(r#""([^"\\]|\\")*""#, |lex| lex.slice()[1..lex.slice().len()-1].replace("\\\"", "\"").replace("\\\\", "\\"))]
    String(String),
    
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse().ok())]
    Number(f64),
    
    #[token("true")]
    True,
    #[token("false")]
    False,
    
    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    // Operators and punctuation
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("=")]
    Equals,
    #[token("->")]
    Arrow,
    #[token("-")]
    Minus,
}

pub struct Parser {
    tokens: Vec<(Token, usize, usize)>, // (token, line, column)
    pos: usize,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let mut lexer = Token::lexer(source);
        let mut tokens = Vec::new();
        let mut line = 1;
        let mut column = 1;
        
        while let Some(token) = lexer.next() {
            match token {
                Ok(tok) => {
                    let col = column;
                    // Estimate column (simplified)
                    column += lexer.slice().len();
                    if lexer.slice().contains('\n') {
                        line += lexer.slice().matches('\n').count();
                        column = 1;
                    }
                    tokens.push((tok, line, col));
                }
                Err(_) => {
                    // Skip invalid tokens for now
                    column += 1;
                }
            }
        }
        
        Self {
            tokens,
            pos: 0,
        }
    }
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _, _)| t)
    }
    
    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].0.clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }
    
    fn expect(&mut self, expected: Token) -> Result<Token> {
        match self.next() {
            Some(tok) => {
                // Check if tokens match (handling data-carrying variants)
                let matches = match (&tok, &expected) {
                    (Token::Module, Token::Module) => true,
                    (Token::Signal, Token::Signal) => true,
                    (Token::Coil, Token::Coil) => true,
                    (Token::Rung, Token::Rung) => true,
                    (Token::When, Token::When) => true,
                    (Token::Then, Token::Then) => true,
                    (Token::Energise, Token::Energise) => true,
                    (Token::DeEnergise, Token::DeEnergise) => true,
                    (Token::NO, Token::NO) => true,
                    (Token::NC, Token::NC) => true,
                    (Token::And, Token::And) => true,
                    (Token::Or, Token::Or) => true,
                    (Token::Not, Token::Not) => true,
                    (Token::Colon, Token::Colon) => true,
                    (Token::Comma, Token::Comma) => true,
                    (Token::LParen, Token::LParen) => true,
                    (Token::RParen, Token::RParen) => true,
                    (Token::Identifier(_), Token::Identifier(_)) => true,
                    (Token::String(_), Token::String(_)) => true,
                    (Token::Number(_), Token::Number(_)) => true,
                    (Token::True, Token::True) => true,
                    (Token::False, Token::False) => true,
                    _ => false,
                };
                
                if matches {
                    Ok(tok)
                } else {
                    let (_, line, col) = self.tokens.get(self.pos - 1).map(|(t, l, c)| (t.clone(), *l, *c)).unwrap_or((Token::Identifier("".to_string()), 1, 1));
                    Err(CompileError::Parse {
                        line,
                        column: col,
                        message: format!("Expected {:?}, found {:?}", expected, tok),
                    })
                }
            }
            None => {
                let (_, line, col) = self.tokens.last().map(|(t, l, c)| (t.clone(), *l, *c)).unwrap_or((Token::Identifier("".to_string()), 1, 1));
                Err(CompileError::Parse {
                    line,
                    column: col,
                    message: format!("Expected {:?}, found end of file", expected),
                })
            }
        }
    }
    
    pub fn parse_module(&mut self) -> Result<Module> {
        self.expect(Token::Module)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err(CompileError::Parse {
                line: 1,
                column: 1,
                message: "Expected module name".to_string(),
            }),
        };
        
        let mut context = None;
        let intent = None;
        let constraints = None;
        let mut signals = Vec::new();
        let mut coils = Vec::new();
        let mut rungs = Vec::new();
        let mut blocks = Vec::new();
        let mut networks = Vec::new();
        
        while let Some(token) = self.peek() {
            match token {
                Token::Context => {
                    self.next();
                    self.expect(Token::Colon)?;
                    if let Some(Token::String(s)) = self.next() {
                        context = Some(s);
                    }
                }
                Token::Signal => {
                    signals.push(self.parse_signal()?);
                }
                Token::Coil => {
                    coils.push(self.parse_coil()?);
                }
                Token::Rung => {
                    rungs.push(self.parse_rung()?);
                }
                Token::Block => {
                    blocks.push(self.parse_block()?);
                }
                Token::Network => {
                    networks.push(self.parse_network()?);
                }
                _ => break,
            }
        }
        
        Ok(Module {
            name,
            context,
            intent,
            constraints,
            signals,
            coils,
            rungs,
            blocks,
            networks,
        })
    }
    
    fn parse_signal(&mut self) -> Result<SignalDecl> {
        self.expect(Token::Signal)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err(CompileError::Parse {
                line: 1,
                column: 1,
                message: "Expected signal name".to_string(),
            }),
        };
        
        let mut parameters = Vec::new();
        if self.peek() == Some(&Token::LParen) {
            self.next();
            while self.peek() != Some(&Token::RParen) {
                if let Some(Token::Identifier(param)) = self.next() {
                    parameters.push(param);
                }
                if self.peek() == Some(&Token::Comma) {
                    self.next();
                } else {
                    break;
                }
            }
            self.expect(Token::RParen)?;
        }
        
        let mut type_ = None;
        if self.peek() == Some(&Token::Colon) {
            self.next();
            if let Some(Token::Identifier(t)) = self.next() {
                type_ = Some(t);
            }
        }
        
        Ok(SignalDecl {
            name,
            parameters,
            type_,
        })
    }
    
    fn parse_coil(&mut self) -> Result<CoilDecl> {
        self.expect(Token::Coil)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err(CompileError::Parse {
                line: 1,
                column: 1,
                message: "Expected coil name".to_string(),
            }),
        };
        
        let mut parameters = Vec::new();
        if self.peek() == Some(&Token::LParen) {
            self.next();
            while self.peek() != Some(&Token::RParen) {
                if let Some(Token::Identifier(param)) = self.next() {
                    parameters.push(param);
                }
                if self.peek() == Some(&Token::Comma) {
                    self.next();
                } else {
                    break;
                }
            }
            self.expect(Token::RParen)?;
        }
        
        let mut latching = None;
        let mut critical = None;
        
        // Parse optional modifiers (simplified - would need more parsing)
        while let Some(token) = self.peek() {
            match token {
                Token::Identifier(ref s) if s == "latching" => {
                    self.next();
                    latching = Some(true);
                }
                Token::Identifier(ref s) if s == "critical" => {
                    self.next();
                    critical = Some(true);
                }
                _ => break,
            }
        }
        
        Ok(CoilDecl {
            name,
            parameters,
            latching,
            critical,
        })
    }
    
    fn parse_rung(&mut self) -> Result<RungDecl> {
        self.expect(Token::Rung)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err(CompileError::Parse {
                line: 1,
                column: 1,
                message: "Expected rung name".to_string(),
            }),
        };
        self.expect(Token::Colon)?;
        self.expect(Token::When)?;
        let guard = self.parse_guard()?;
        self.expect(Token::Then)?;
        let actions = self.parse_actions()?;
        
        Ok(RungDecl {
            name,
            guard,
            actions,
        })
    }
    
    fn parse_guard(&mut self) -> Result<GuardExpr> {
        self.parse_guard_or()
    }
    
    fn parse_guard_or(&mut self) -> Result<GuardExpr> {
        let mut left = self.parse_guard_and()?;
        while self.peek() == Some(&Token::Or) {
            self.next();
            let right = self.parse_guard_and()?;
            left = GuardExpr::Or {
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }
    
    fn parse_guard_and(&mut self) -> Result<GuardExpr> {
        let mut left = self.parse_guard_not()?;
        while self.peek() == Some(&Token::And) {
            self.next();
            let right = self.parse_guard_not()?;
            left = GuardExpr::And {
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }
    
    fn parse_guard_not(&mut self) -> Result<GuardExpr> {
        if self.peek() == Some(&Token::Not) {
            self.next();
            let expr = self.parse_guard_primary()?;
            Ok(GuardExpr::Not {
                expr: Box::new(expr),
            })
        } else {
            self.parse_guard_primary()
        }
    }
    
    fn parse_guard_primary(&mut self) -> Result<GuardExpr> {
        if self.peek() == Some(&Token::LParen) {
            self.next();
            let expr = self.parse_guard()?;
            self.expect(Token::RParen)?;
            Ok(expr)
        } else if self.peek() == Some(&Token::NO) || self.peek() == Some(&Token::NC) {
            let contact_type = match self.next() {
                Some(Token::NO) => ContactType::NO,
                Some(Token::NC) => ContactType::NC,
                _ => unreachable!(),
            };
            let name = match self.next() {
                Some(Token::Identifier(name)) => name,
                _ => return Err(CompileError::Parse {
                    line: 1,
                    column: 1,
                    message: "Expected signal/coil name after NO/NC".to_string(),
                }),
            };
            
            let mut arguments = Vec::new();
            if self.peek() == Some(&Token::LParen) {
                self.next();
                while self.peek() != Some(&Token::RParen) {
                    arguments.push(self.parse_expr()?);
                    if self.peek() == Some(&Token::Comma) {
                        self.next();
                    } else {
                        break;
                    }
                }
                self.expect(Token::RParen)?;
            }
            
            Ok(GuardExpr::Contact {
                name,
                contact_type,
                arguments,
            })
        } else {
            // Bare identifier (treated as NO contact)
            let name = match self.next() {
                Some(Token::Identifier(name)) => name,
                _ => return Err(CompileError::Parse {
                    line: 1,
                    column: 1,
                    message: "Expected contact or identifier".to_string(),
                }),
            };
            Ok(GuardExpr::Contact {
                name,
                contact_type: ContactType::NO,
                arguments: Vec::new(),
            })
        }
    }
    
    fn parse_expr(&mut self) -> Result<Expr> {
        match self.next() {
            Some(Token::String(s)) => Ok(Expr::String(s)),
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::True) => Ok(Expr::Boolean(true)),
            Some(Token::False) => Ok(Expr::Boolean(false)),
            Some(Token::Identifier(name)) => Ok(Expr::Identifier(name)),
            _ => Err(CompileError::Parse {
                line: 1,
                column: 1,
                message: "Expected expression".to_string(),
            }),
        }
    }
    
    fn parse_actions(&mut self) -> Result<Vec<Action>> {
        let mut actions = Vec::new();
        loop {
            let action = match self.peek() {
                Some(Token::Energise) => {
                    self.next();
                    let coil = match self.next() {
                        Some(Token::Identifier(name)) => name,
                        _ => return Err(CompileError::Parse {
                            line: 1,
                            column: 1,
                            message: "Expected coil name".to_string(),
                        }),
                    };
                    let mut arguments = Vec::new();
                    if self.peek() == Some(&Token::LParen) {
                        self.next();
                        while self.peek() != Some(&Token::RParen) {
                            arguments.push(self.parse_expr()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.next();
                            } else {
                                break;
                            }
                        }
                        self.expect(Token::RParen)?;
                    }
                    Action {
                        action_type: ActionType::Energise,
                        coil,
                        arguments,
                    }
                }
                Some(Token::DeEnergise) => {
                    self.next();
                    let coil = match self.next() {
                        Some(Token::Identifier(name)) => name,
                        _ => return Err(CompileError::Parse {
                            line: 1,
                            column: 1,
                            message: "Expected coil name".to_string(),
                        }),
                    };
                    Action {
                        action_type: ActionType::DeEnergise,
                        coil,
                        arguments: Vec::new(),
                    }
                }
                _ => break,
            };
            actions.push(action);
        }
        Ok(actions)
    }
    
    fn parse_block(&mut self) -> Result<BlockDecl> {
        self.expect(Token::Block)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err(CompileError::Parse {
                line: 1,
                column: 1,
                message: "Expected block name".to_string(),
            }),
        };
        self.expect(Token::Colon)?;
        
        // Simplified block parsing - would need full implementation
        Ok(BlockDecl {
            name,
            inputs: Vec::new(),
            outputs: Vec::new(),
            internals: Vec::new(),
            implementation: None,
            effect: None,
        })
    }
    
    fn parse_network(&mut self) -> Result<NetworkDecl> {
        self.expect(Token::Network)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err(CompileError::Parse {
                line: 1,
                column: 1,
                message: "Expected network name".to_string(),
            }),
        };
        self.expect(Token::Colon)?;
        
        // Simplified network parsing
        Ok(NetworkDecl {
            name,
            wires: Vec::new(),
            outputs: Vec::new(),
        })
    }
}

pub fn parse(source: &str) -> Result<Module> {
    let mut parser = Parser::new(source);
    parser.parse_module()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_module() {
        let source = r#"
module test_module

signal input_signal
coil output_coil

rung test_rung:
  when NO input_signal
  then energise output_coil
"#;
        let result = parse(source);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name, "test_module");
        assert_eq!(module.signals.len(), 1);
        assert_eq!(module.coils.len(), 1);
        assert_eq!(module.rungs.len(), 1);
    }
}
