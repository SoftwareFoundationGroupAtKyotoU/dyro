#![allow(dead_code)]

use crate::ast;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ANFVar(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ANFType(pub ast::ASTType);

impl ANFType {
    pub fn size(&self) -> usize {
        use ast::ASTType::*;

        match self.0 {
            // Don't want to consider ZST now
            Unit => 1,
            Int => 4,
            Bool => 1,
            Tuple(ref types) => types.iter().map(|t| ANFType(t.clone()).size()).sum(),

            // [heap/stack enum, size, value]
            MutPtr(_) => 9,

            // We probably can't store these types in memory so 0
            String => 0,
            Function(_, _) => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ANFUnaryOp(pub ast::ASTUnaryOp);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ANFBinaryOp(pub ast::ASTBinaryOp);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ANFNode {
    Let {
        binding: ANFVar,
        value: ANFSimpleExpression,
        body: Box<ANFNode>,
    },
    LetFn {
        name: ANFVar,
        args: Vec<(ANFVar, ANFType)>,
        return_type: ANFType,
        value: Box<ANFNode>,
        body: Box<ANFNode>,
    },
    Simple(ANFSimpleExpression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ANFSimpleExpression {
    Unit,
    Int(i32),
    Bool(bool),
    Var(ANFVar),
    String(String),
    SpecialFunction(crate::special::SpecialFunction),
    If {
        condition: ANFVar,
        // We need to use ANFSimpleExpression here to support function call (it could be a continuation here?)
        then: Box<ANFSimpleExpression>,
        r#else: Box<ANFSimpleExpression>,
    },
    Call {
        function: ANFVar,
        type_arguments: Vec<ANFType>,
        arguments: Vec<ANFVar>,
    },
    TupleAccess {
        tuple: ANFVar,
        index: usize,
    },
    Tuple {
        elements: Vec<ANFVar>,
    },
    ArrayRead {
        array: ANFVar,
        index: ANFVar,
    },
    ArraySet {
        array: ANFVar,
        index: ANFVar,
        value: ANFVar,
    },
    UnaryOp {
        op: ANFUnaryOp,
        operand: ANFVar,
    },
    BinaryOp {
        op: ANFBinaryOp,
        left: ANFVar,
        right: ANFVar,
    },
}
