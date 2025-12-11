/// Abstract Syntax Tree for Charta programs

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub context: Option<String>,
    pub intent: Option<Intent>,
    pub constraints: Option<Constraints>,
    pub signals: Vec<SignalDecl>,
    pub coils: Vec<CoilDecl>,
    pub rungs: Vec<RungDecl>,
    pub blocks: Vec<BlockDecl>,
    pub networks: Vec<NetworkDecl>,
}

#[derive(Debug, Clone)]
pub struct Intent {
    pub goal: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Constraints {
    pub data_privacy: Option<DataPrivacy>,
    pub quality: Option<Quality>,
    pub cost: Option<Cost>,
}

#[derive(Debug, Clone)]
pub struct DataPrivacy {
    pub jurisdiction: Option<String>,
    pub pii_handling: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Quality {
    pub min_precision: Option<f64>,
    pub min_recall: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Cost {
    pub max_cost_per_submission: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SignalDecl {
    pub name: String,
    pub parameters: Vec<String>,
    pub type_: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CoilDecl {
    pub name: String,
    pub parameters: Vec<String>,
    pub latching: Option<bool>,
    pub critical: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct RungDecl {
    pub name: String,
    pub guard: GuardExpr,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone)]
pub enum GuardExpr {
    Contact {
        name: String,
        contact_type: ContactType,
        arguments: Vec<Expr>,
    },
    And {
        left: Box<GuardExpr>,
        right: Box<GuardExpr>,
    },
    Or {
        left: Box<GuardExpr>,
        right: Box<GuardExpr>,
    },
    Not {
        expr: Box<GuardExpr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactType {
    NO, // Normally Open
    NC, // Normally Closed
}

#[derive(Debug, Clone)]
pub enum Expr {
    String(String),
    Number(f64),
    Boolean(bool),
    Identifier(String),
}

#[derive(Debug, Clone)]
pub struct Action {
    pub action_type: ActionType,
    pub coil: String,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Energise,
    DeEnergise,
    Escalate,
    Require,
}

#[derive(Debug, Clone)]
pub struct BlockDecl {
    pub name: String,
    pub inputs: Vec<PortDecl>,
    pub outputs: Vec<PortDecl>,
    pub internals: Vec<InternalDecl>,
    pub implementation: Option<String>,
    pub effect: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PortDecl {
    pub name: String,
    pub type_: String,
}

#[derive(Debug, Clone)]
pub struct InternalDecl {
    pub name: String,
    pub type_: String,
}

#[derive(Debug, Clone)]
pub struct NetworkDecl {
    pub name: String,
    pub wires: Vec<Wire>,
    pub outputs: Vec<Output>,
}

#[derive(Debug, Clone)]
pub struct Wire {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone)]
pub struct Output {
    pub name: String,
    pub source: String,
}
