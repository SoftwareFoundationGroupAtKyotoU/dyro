#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ASTVar(pub String);

impl From<&str> for ASTVar {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ASTType {
    Unit,
    Int,
    Bool,
    String,
    MutPtr(Box<ASTType>),
    Tuple(Vec<ASTType>),
    Function(Vec<ASTType>, Box<ASTType>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ASTUnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ASTBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ASTNode {
    Let {
        binding: ASTVar,
        value: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    LetFn {
        name: ASTVar,
        args: Vec<(ASTVar, ASTType)>,
        return_type: ASTType,
        value: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    Unit,
    Int(i32),
    Bool(bool),
    Var(ASTVar),
    /// For panic call only
    String(String),
    SpecialFunction(crate::special::SpecialFunction),
    If {
        condition: Box<ASTNode>,
        then: Box<ASTNode>,
        r#else: Box<ASTNode>,
        r#type: ASTType,
    },
    /// Partial evaluation is not supported
    Call {
        function: Box<ASTNode>,
        type_arguments: Vec<ASTType>,
        arguments: Vec<ASTNode>,
    },
    TupleAccess {
        tuple: Box<ASTNode>,
        index: usize,
    },
    Tuple {
        elements: Vec<ASTNode>,
    },
    /// array[index]
    ArrayRead {
        array: Box<ASTNode>,
        index: Box<ASTNode>,
    },
    /// array[index] = value
    ArraySet {
        array: Box<ASTNode>,
        index: Box<ASTNode>,
        value: Box<ASTNode>,
    },
    /// (first; second) equivalent to let _ = first in second
    /// removed in ANF
    Sequence {
        first: Box<ASTNode>,
        second: Box<ASTNode>,
    },
    UnaryOp {
        op: ASTUnaryOp,
        operand: Box<ASTNode>,
    },
    BinaryOp {
        op: ASTBinaryOp,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
}
