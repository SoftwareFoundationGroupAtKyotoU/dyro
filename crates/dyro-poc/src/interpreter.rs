use crate::{
    anf::{self, ANFType},
    ast::{self, ASTType},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtrLocation {
    Stack(u32),
    Heap(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    String(String),
    Bool(bool),
    Tuple(Vec<Value>),
    SpecialFunction(crate::special::SpecialFunction),
    MutPtr {
        location: PtrLocation,
        r#type: anf::ANFType,
    },
    Function {
        args: Vec<(anf::ANFVar, anf::ANFType)>,
        return_type: anf::ANFType,
        value: anf::ANFNode,
    },
}

impl Value {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Value::Int(i) => i.to_le_bytes().to_vec(),
            Value::Bool(b) => vec![*b as u8],
            Value::Tuple(elements) => elements.iter().flat_map(Value::to_bytes).collect(),
            Value::MutPtr { location, .. } => match location {
                PtrLocation::Stack(offset) => {
                    [vec![1, 0, 0, 0], offset.to_le_bytes().to_vec()].concat()
                }
                PtrLocation::Heap(offset) => {
                    [vec![2, 0, 0, 0], offset.to_le_bytes().to_vec()].concat()
                }
            },
            Value::SpecialFunction(_) => panic!("SpecialFunction cannot be converted to bytes"),
            Value::String(_) => panic!("String cannot be converted to bytes"),
            Value::Function { .. } => panic!("Function cannot be converted to bytes"),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, r#type: ANFType) -> anyhow::Result<Self> {
        use ast::ASTType::*;
        let type_size = r#type.size();
        match (r#type.0, bytes.len()) {
            (Int, 4) => Ok(Value::Int(i32::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
            ]))),
            (Bool, 1) => Ok(Value::Bool(bytes[0] != 0)),
            (Tuple(types), l) if type_size == l => {
                let mut elements = Vec::new();
                let mut offset = 0;
                for t in types {
                    let size = ANFType(t.clone()).size();
                    elements.push(Value::from_bytes(
                        bytes[offset..offset + size].to_vec(),
                        anf::ANFType(t),
                    )?);
                    offset += size;
                }
                Ok(Value::Tuple(elements))
            }
            (MutPtr(t), 8) => {
                let location = match [bytes[0], bytes[1], bytes[2], bytes[3]] {
                    [1, 0, 0, 0] => PtrLocation::Stack(u32::from_le_bytes([
                        bytes[4], bytes[5], bytes[6], bytes[7],
                    ])),
                    [2, 0, 0, 0] => PtrLocation::Heap(u32::from_le_bytes([
                        bytes[4], bytes[5], bytes[6], bytes[7],
                    ])),
                    _ => return Err(anyhow::anyhow!("Invalid location")),
                };
                Ok(Value::MutPtr {
                    location,
                    r#type: anf::ANFType(*t),
                })
            }
            (String | Function(_, _), _) => Err(anyhow::anyhow!("Invalid type for bytes")),
            _ => Err(anyhow::anyhow!("Invalid bytes")),
        }
    }
}

impl Value {
    pub fn anf_type(&self) -> anf::ANFType {
        use ast::ASTType::*;

        anf::ANFType(match self {
            Value::Int(_) => Int,
            Value::String(_) => String,
            Value::Bool(_) => Bool,
            Value::Tuple(elements) => {
                Tuple(elements.iter().map(Value::anf_type).map(|t| t.0).collect())
            }
            Value::MutPtr { r#type, .. } => MutPtr(Box::new(r#type.clone().0)),
            Value::Function {
                args,
                return_type,
                value: _,
            } => Function(
                args.iter().map(|(_, t)| t.0.clone()).collect(),
                Box::new(return_type.clone().0),
            ),
            Value::SpecialFunction(_) => unreachable!(),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HeapValue {
    Byte(u8),
    Poison,
    Undefined,
}

impl HeapValue {
    pub fn as_byte(&self) -> anyhow::Result<u8> {
        match self {
            HeapValue::Byte(b) => Ok(*b),
            HeapValue::Poison => Err(anyhow::anyhow!("Poison value")),
            HeapValue::Undefined => Err(anyhow::anyhow!("Undefined value")),
        }
    }
}

pub enum RootOrParent {
    Root { heap: Vec<HeapValue> },
    Parent(Box<Interpreter>),
}

pub struct Interpreter {
    pub parent: RootOrParent,
    pub stack: Vec<Option<Value>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            parent: RootOrParent::Root { heap: Vec::new() },
            stack: Vec::new(),
        }
    }

    pub fn ensure_stack(&mut self, var: anf::ANFVar) {
        while self.stack.len() <= var.0 as usize {
            self.stack.push(None);
        }
    }

    pub fn alloc(&mut self, size: usize) -> PtrLocation {
        match &mut self.parent {
            RootOrParent::Root { heap } => {
                let offset = heap.len();
                heap.resize(offset + size, HeapValue::Poison);
                PtrLocation::Heap(offset as u32)
            }
            RootOrParent::Parent(parent) => parent.alloc(size),
        }
    }

    pub fn read_heap(&self, offset: usize) -> anyhow::Result<u8> {
        match &self.parent {
            RootOrParent::Root { ref heap } => heap
                .get(offset)
                .map(HeapValue::as_byte)
                .transpose()?
                .ok_or_else(|| anyhow::anyhow!("Invalid heap access")),
            RootOrParent::Parent(parent) => parent.read_heap(offset),
        }
    }

    pub fn write_heap(&mut self, offset: usize, byte: u8) -> anyhow::Result<()> {
        match &mut self.parent {
            RootOrParent::Root { heap } => {
                while heap.len() <= offset {
                    heap.push(HeapValue::Undefined);
                }
                heap[offset] = HeapValue::Byte(byte);
                Ok(())
            }
            RootOrParent::Parent(parent) => parent.write_heap(offset, byte),
        }
    }

    pub fn calc_unary_op(&self, op: anf::ANFUnaryOp, value: Value) -> anyhow::Result<Value> {
        use crate::ast::ASTUnaryOp::*;
        match op.0 {
            Neg => match value {
                Value::Int(i) => Ok(Value::Int(-i)),
                _ => Err(anyhow::anyhow!("Invalid type for Neg")),
            },
            Not => match value {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(anyhow::anyhow!("Invalid type for Not")),
            },
        }
    }

    pub fn calc_binary_op(
        &self,
        op: anf::ANFBinaryOp,
        left: Value,
        right: Value,
    ) -> anyhow::Result<Value> {
        use crate::ast::ASTBinaryOp::*;
        match op.0 {
            Add => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
                _ => Err(anyhow::anyhow!("Invalid types for Add")),
            },
            Sub => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
                _ => Err(anyhow::anyhow!("Invalid types for Sub")),
            },
            Mul => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
                _ => Err(anyhow::anyhow!("Invalid types for Mul")),
            },
            Div => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l / r)),
                _ => Err(anyhow::anyhow!("Invalid types for Div")),
            },
            Eq => Ok(Value::Bool(left == right)),
            Lt => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l < r)),
                _ => Err(anyhow::anyhow!("Invalid types for Lt")),
            },
            Gt => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l > r)),
                _ => Err(anyhow::anyhow!("Invalid types for Gt")),
            },
            Le => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l <= r)),
                _ => Err(anyhow::anyhow!("Invalid types for Le")),
            },
            Ge => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l >= r)),
                _ => Err(anyhow::anyhow!("Invalid types for Ge")),
            },
        }
    }

    pub fn eval_var(&self, var: anf::ANFVar) -> anyhow::Result<Value> {
        match self.stack.get(var.0 as usize).cloned().flatten() {
            Some(value) => Ok(value),
            None => match &self.parent {
                RootOrParent::Parent(parent) => parent.eval_var(var),
                RootOrParent::Root { .. } => Err(anyhow::anyhow!("Variable {} not found", var.0)),
            },
        }
    }

    pub fn call_special_function(
        &mut self,
        special_function: crate::special::SpecialFunction,
        type_arguments: Vec<anf::ANFType>,
        arguments: Vec<anf::ANFVar>,
    ) -> anyhow::Result<Value> {
        use crate::special::SpecialFunction::*;

        match (special_function, &type_arguments[..], &arguments[..]) {
            (Alloc, [r#type], [length]) => {
                let type_size = r#type.size();
                let alloc_length: usize = if let Value::Int(i) = self.eval_var(*length)? {
                    i.try_into()
                        .map_err(|_| anyhow::anyhow!("Invalid length"))?
                } else {
                    return Err(anyhow::anyhow!("Invalid argument for Alloc"));
                };
                let location = self.alloc(type_size * alloc_length);

                Ok(Value::MutPtr {
                    location,
                    r#type: r#type.clone(),
                })
            }
            (Panic, [], [var]) => {
                let message = self.eval_var(*var)?;
                if let Value::String(s) = message {
                    Err(anyhow::anyhow!("Panic: {}", s))
                } else {
                    Err(anyhow::anyhow!("Invalid argument {:?} for Panic", message))
                }
            }
            (function, _, _) => anyhow::bail!("Invalid arguments for {:?}", function),
        }
    }

    pub fn eval_simple_expression<'a>(
        &'a mut self,
        expr: &anf::ANFSimpleExpression,
    ) -> anyhow::Result<Value> {
        match expr {
            anf::ANFSimpleExpression::Int(i) => Ok(Value::Int(*i)),
            anf::ANFSimpleExpression::Bool(b) => Ok(Value::Bool(*b)),
            anf::ANFSimpleExpression::Var(v) => self.eval_var(*v),
            anf::ANFSimpleExpression::String(s) => Ok(Value::String(s.clone())),
            anf::ANFSimpleExpression::SpecialFunction(special_function) => {
                Ok(Value::SpecialFunction(*special_function))
            }
            anf::ANFSimpleExpression::BinaryOp { op, left, right } => {
                self.calc_binary_op(*op, self.eval_var(*left)?, self.eval_var(*right)?)
            }
            anf::ANFSimpleExpression::UnaryOp { op, operand } => {
                self.calc_unary_op(*op, self.eval_var(*operand)?)
            }
            anf::ANFSimpleExpression::If {
                condition,
                then,
                r#else,
            } => {
                let condition = self.eval_var(*condition)?;
                match condition {
                    Value::Bool(true) => self.eval_simple_expression(then),
                    Value::Bool(false) => self.eval_simple_expression(r#else),
                    _ => Err(anyhow::anyhow!("Invalid type for If")),
                }
            }
            anf::ANFSimpleExpression::Call {
                function,
                type_arguments,
                arguments,
            } => {
                let function = self.eval_var(*function)?;
                match function {
                    Value::SpecialFunction(special_function) => self.call_special_function(
                        special_function,
                        type_arguments.clone(),
                        arguments.clone(),
                    ),
                    Value::Function {
                        args,
                        value,
                        return_type,
                    } => {
                        anyhow::ensure!(
                            args.len() == arguments.len(),
                            "Invalid number of arguments"
                        );

                        let mut new_interpreter = Interpreter {
                            parent: RootOrParent::Parent(Box::new(std::mem::replace(
                                self,
                                Interpreter::new(),
                            ))),
                            stack: Vec::new(),
                        };

                        for ((var, r#type), value) in args.iter().zip(arguments.iter()) {
                            new_interpreter.ensure_stack(*var);
                            new_interpreter.stack[var.0 as usize] =
                                Some(new_interpreter.eval_var(*value)?);
                            anyhow::ensure!(
                                &new_interpreter.stack[var.0 as usize]
                                    .as_ref()
                                    .unwrap()
                                    .anf_type()
                                    == r#type,
                                "Invalid argument"
                            );
                        }

                        let result = new_interpreter.eval_node(&value)?;
                        anyhow::ensure!(result.anf_type() == return_type, "Invalid return type");

                        match new_interpreter.parent {
                            RootOrParent::Parent(parent) => {
                                *self = *parent;
                            }
                            _ => panic!("Invalid parent"),
                        }

                        Ok(result)
                    }
                    _ => Err(anyhow::anyhow!("Invalid type for Call")),
                }
            }
            anf::ANFSimpleExpression::TupleAccess { tuple, index } => {
                match self.eval_var(*tuple)? {
                    Value::Tuple(elements) => elements
                        .get(*index)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("Index out of bounds")),
                    _ => Err(anyhow::anyhow!("Invalid type for TupleAccess")),
                }
            }
            anf::ANFSimpleExpression::Tuple { elements } => elements
                .iter()
                .map(|var| self.eval_var(*var))
                .collect::<anyhow::Result<Vec<Value>>>()
                .map(Value::Tuple),
            anf::ANFSimpleExpression::ArrayRead { array, index } => {
                let array = self.eval_var(*array)?;
                let index = self.eval_var(*index)?;

                match (array, index) {
                    (Value::MutPtr { location, r#type }, Value::Int(index)) => {
                        let offset = match location {
                            PtrLocation::Stack(_offset) => unimplemented!(),
                            PtrLocation::Heap(offset) => offset as usize,
                        };
                        let type_size = r#type.size();
                        let bytes = (0..type_size)
                            .map(|i| self.read_heap(offset + index as usize * type_size + i))
                            .collect::<anyhow::Result<Vec<u8>>>()?;
                        Value::from_bytes(bytes, r#type)
                    }
                    _ => Err(anyhow::anyhow!("Invalid types for ArrayRead")),
                }
            }
            anf::ANFSimpleExpression::ArraySet {
                array,
                index,
                value,
            } => {
                let array = self.eval_var(*array)?;
                let index = self.eval_var(*index)?;
                let value = self.eval_var(*value)?;

                match (array, index, value) {
                    (Value::MutPtr { location, r#type }, Value::Int(index), value) => {
                        let offset = match location {
                            PtrLocation::Stack(_offset) => unimplemented!(),
                            PtrLocation::Heap(offset) => offset as usize,
                        };
                        let type_size = r#type.size();
                        let bytes = value.to_bytes();
                        anyhow::ensure!(bytes.len() == type_size, "Invalid size for ArraySet");
                        for (i, byte) in bytes.into_iter().enumerate() {
                            self.write_heap(offset + index as usize * type_size + i, byte)?;
                        }
                        Ok(Value::Tuple(Vec::new()))
                    }
                    _ => Err(anyhow::anyhow!("Invalid types for ArraySet")),
                }
            }
        }
    }

    pub fn eval_node(&mut self, node: &anf::ANFNode) -> anyhow::Result<Value> {
        match node {
            anf::ANFNode::Let {
                binding,
                value,
                body,
            } => {
                self.ensure_stack(*binding);
                self.stack[binding.0 as usize] = Some(self.eval_simple_expression(value)?);
                self.eval_node(body)
            }

            anf::ANFNode::LetFn {
                name,
                args,
                return_type,
                value,
                body,
            } => {
                self.ensure_stack(*name);
                self.stack[name.0 as usize] = Some(Value::Function {
                    args: args.clone(),
                    return_type: return_type.clone(),
                    value: (**value).clone(),
                });
                self.eval_node(body)
            }

            anf::ANFNode::Simple(expr) => self.eval_simple_expression(expr),
        }
    }
}
