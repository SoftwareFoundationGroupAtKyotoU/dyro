pub mod anf;
pub mod ast;
pub mod ast_to_anf;
pub mod interpreter;
pub mod special;

#[macro_export]
macro_rules! seq {
    ($a:expr) => {
        $a
    };
    ($a:expr; $($rest:expr);+) => {
        Sequence {
            first: Box::new($a),
            second: Box::new(seq!($($rest);+))
        }
    };
}

#[macro_export]
macro_rules! ast {
    ($x:expr) => {{
        use dyro_poc::ast::{ASTBinaryOp::*, ASTNode::*, ASTUnaryOp::*};
        type T = dyro_poc::ast::ASTType;
        $x
    }};
}
