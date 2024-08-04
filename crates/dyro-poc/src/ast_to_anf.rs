use crate::{anf, ast, special};

struct ASTToANFConverter {
    var_counter: u32,
}

impl ASTToANFConverter {
    fn new() -> Self {
        Self { var_counter: 0 }
    }

    fn new_var(&mut self) -> anf::ANFVar {
        let var = self.var_counter;
        self.var_counter += 1;
        anf::ANFVar(var)
    }

    fn convert(
        &mut self,
        context: im::HashMap<ast::ASTVar, anf::ANFVar>,
        value: &ast::ASTNode,
    ) -> anyhow::Result<anf::ANFNode> {
        let dummy_var = self.new_var();
        self.convert_bind_into(
            context,
            dummy_var,
            value,
            anf::ANFNode::Simple(anf::ANFSimpleExpression::Var(dummy_var)),
        )
    }

    fn convert_bind_into(
        &mut self,
        context: im::HashMap<ast::ASTVar, anf::ANFVar>,
        binding: anf::ANFVar,
        value: &ast::ASTNode,
        body: anf::ANFNode,
    ) -> anyhow::Result<anf::ANFNode> {
        #[cfg(feature = "debug-ast-to-anf")]
        let context_backup = context.clone();
        #[cfg(feature = "debug-ast-to-anf")]
        let body_backup = body.clone();

        let result = match value {
            ast::ASTNode::Let {
                binding: let_binding,
                value: let_value,
                body: let_body,
            } => {
                let let_value_var = self.new_var();
                let let_body_var = binding;

                let mut let_body_context = context.clone();
                let_body_context.insert(let_binding.clone(), let_value_var);
                let result =
                    self.convert_bind_into(let_body_context, let_body_var, let_body, body)?;

                self.convert_bind_into(context, let_value_var, let_value, result)
            }

            ast::ASTNode::LetFn {
                name: fn_name,
                args: fn_args,
                return_type: fn_return_type,
                value: fn_value,
                body: fn_body,
            } => {
                // Before:
                //   let <binding> = (let <fn_var> x = <fn_value> in <fn_body>) in <body>
                //
                // After:
                //   let <fn_var> x = <fn_value> in let <fn_body_var'> = <fn_body> in
                //   let <binding> = <fn_body_var'> in <body>
                let fn_var = self.new_var();
                let fn_body_var = binding;

                // allow rec by default
                let mut fn_body_context = context.clone();
                fn_body_context.insert(fn_name.clone(), fn_var);

                let mut fn_value_context = context.clone();
                let fn_anf_args = fn_args
                    .iter()
                    .map(|(arg_name, arg_type)| {
                        let arg_var = self.new_var();
                        fn_value_context.insert(arg_name.clone(), arg_var.clone());
                        (arg_var, anf::ANFType(arg_type.clone()))
                    })
                    .collect::<Vec<_>>();

                let result = anf::ANFNode::Let {
                    binding: binding,
                    value: anf::ANFSimpleExpression::Var(fn_body_var),
                    body: Box::new(body),
                };
                let result =
                    self.convert_bind_into(fn_body_context, fn_body_var, fn_body, result)?;

                Ok(anf::ANFNode::LetFn {
                    name: fn_var,
                    args: fn_anf_args,
                    return_type: anf::ANFType(fn_return_type.clone()),
                    value: Box::new(self.convert(fn_value_context, fn_value)?),
                    body: Box::new(result),
                })
            }

            ast::ASTNode::Int(i) => {
                let value = anf::ANFSimpleExpression::Int(*i);
                Ok(anf::ANFNode::Let {
                    binding,
                    value,
                    body: Box::new(body),
                })
            }

            ast::ASTNode::Bool(b) => {
                let value = anf::ANFSimpleExpression::Bool(*b);
                Ok(anf::ANFNode::Let {
                    binding,
                    value,
                    body: Box::new(body),
                })
            }

            ast::ASTNode::Var(v) => {
                let value = anf::ANFSimpleExpression::Var(
                    context
                        .get(&v)
                        .ok_or_else(|| anyhow::anyhow!("Variable {} not found in context", v.0))?
                        .clone(),
                );
                Ok(anf::ANFNode::Let {
                    binding,
                    value,
                    body: Box::new(body),
                })
            }

            ast::ASTNode::String(s) => {
                let value = anf::ANFSimpleExpression::String(s.clone());
                Ok(anf::ANFNode::Let {
                    binding,
                    value,
                    body: Box::new(body),
                })
            }

            ast::ASTNode::If {
                condition,
                then,
                r#else,
                r#type,
            } => {
                // Before:
                //   let <binding> = if <condition> then <then> else <r#else> in <body>
                // After:
                //   let <then_var> () = <then> in
                //   let <else_var> () = <r#else> in
                //   let <condition_var> = <condition> in
                //   let <binding> = if <condition_var> then <then_var> () else <else_var> () in <body>

                let condition_var = self.new_var();
                let then_var = self.new_var();
                let else_var = self.new_var();

                let value = anf::ANFSimpleExpression::If {
                    condition: condition_var,
                    then: Box::new(anf::ANFSimpleExpression::Call {
                        function: then_var,
                        type_arguments: vec![],
                        arguments: vec![],
                    }),
                    r#else: Box::new(anf::ANFSimpleExpression::Call {
                        function: else_var,
                        type_arguments: vec![],
                        arguments: vec![],
                    }),
                };

                let result = anf::ANFNode::Let {
                    binding,
                    value,
                    body: Box::new(body),
                };

                let result =
                    self.convert_bind_into(context.clone(), condition_var, condition, result)?;

                let result = anf::ANFNode::LetFn {
                    name: else_var,
                    args: vec![],
                    return_type: anf::ANFType(r#type.clone()),
                    value: Box::new(self.convert(context.clone(), r#else)?),
                    body: Box::new(result),
                };

                let result = anf::ANFNode::LetFn {
                    name: then_var,
                    args: vec![],
                    return_type: anf::ANFType(r#type.clone()),
                    value: Box::new(self.convert(context.clone(), then)?),
                    body: Box::new(result),
                };

                Ok(result)
            }

            ast::ASTNode::Call {
                function,
                type_arguments,
                arguments,
            } => {
                let type_arguments = type_arguments
                    .iter()
                    .map(|t| anf::ANFType(t.clone()))
                    .collect();

                let function_var = self.new_var();
                let arguments_vars = arguments
                    .iter()
                    .map(|arg| self.new_var())
                    .collect::<Vec<_>>();

                let mut result = anf::ANFNode::Let {
                    binding,
                    value: anf::ANFSimpleExpression::Call {
                        function: function_var,
                        type_arguments,
                        arguments: arguments_vars.clone(),
                    },
                    body: Box::new(body),
                };

                for (index, arg) in arguments.iter().enumerate().rev() {
                    result = self.convert_bind_into(
                        context.clone(),
                        arguments_vars[index],
                        arg,
                        result,
                    )?;
                }

                self.convert_bind_into(context, function_var, function, result)
            }

            ast::ASTNode::TupleAccess { tuple, index } => {
                // tuple[index]
                // -> let tuple_var = tuple in let index_var = index in tuple_var[index_var]
                let tuple_var = self.new_var();

                let value = anf::ANFSimpleExpression::TupleAccess {
                    tuple: tuple_var,
                    index: *index,
                };

                let result = anf::ANFNode::Let {
                    binding,
                    value,
                    body: Box::new(body),
                };

                self.convert_bind_into(context, tuple_var, tuple, result)
            }

            ast::ASTNode::Tuple { elements } => {
                // (e1, e2, e3)
                // -> let e1_var = e1 in let e2_var = e2 in let e3_var = e3 in (e1_var, e2_var, e3_var)
                let mut vars = Vec::new();
                for _ in elements.iter() {
                    vars.push(self.new_var());
                }

                let mut result = anf::ANFNode::Let {
                    binding,
                    value: anf::ANFSimpleExpression::Tuple {
                        elements: vars.clone(),
                    },
                    body: Box::new(body),
                };

                for (index, element) in elements.iter().enumerate().rev() {
                    result =
                        self.convert_bind_into(context.clone(), vars[index], element, result)?;
                }

                Ok(result)
            }

            ast::ASTNode::ArrayRead { array, index } => {
                // Before:
                //   let <binding> = <array>[<index>] in <body>
                // After:
                //   let <array_var> = <array> in
                //   let <index_var> = <index> in
                //   let <binding> = <array_var>[<index_var>] in <body>
                let array_var = self.new_var();
                let index_var = self.new_var();

                let result = anf::ANFNode::Let {
                    binding,
                    value: anf::ANFSimpleExpression::ArrayRead {
                        array: array_var,
                        index: index_var,
                    },
                    body: Box::new(body),
                };

                let result = self.convert_bind_into(context.clone(), index_var, index, result)?;
                self.convert_bind_into(context, array_var, array, result)
            }

            ast::ASTNode::ArraySet {
                array,
                index,
                value,
            } => {
                // Before:
                //   let <binding> = (<array>[<index>] := <value>) in <body>
                // After:
                //   let <array_var> = <array> in
                //   let <index_var> = <index> in
                //   let <value_var> = <value> in
                //   let <binding> = <array_var>[<index_var>] := <value_var> in <body>

                let array_var = self.new_var();
                let index_var = self.new_var();
                let value_var = self.new_var();

                let result = anf::ANFNode::Let {
                    binding,
                    value: anf::ANFSimpleExpression::ArraySet {
                        array: array_var,
                        index: index_var,
                        value: value_var,
                    },
                    body: Box::new(body),
                };

                let result = self.convert_bind_into(context.clone(), value_var, value, result)?;
                let result = self.convert_bind_into(context.clone(), index_var, index, result)?;
                self.convert_bind_into(context, array_var, array, result)
            }

            ast::ASTNode::Sequence { first, second } => {
                let second_var = self.new_var();
                let second = self.convert_bind_into(
                    context.clone(),
                    second_var,
                    &second,
                    anf::ANFNode::Simple(anf::ANFSimpleExpression::Var(second_var)),
                )?;

                let first_dummy_var = self.new_var();
                self.convert_bind_into(context, first_dummy_var, &first, second)
            }

            ast::ASTNode::UnaryOp { op, operand } => {
                let operand_var = self.new_var();

                let remaining = anf::ANFNode::Simple(anf::ANFSimpleExpression::UnaryOp {
                    op: anf::ANFUnaryOp(op.clone()),
                    operand: operand_var,
                });

                self.convert_bind_into(context, operand_var, &operand, remaining)
            }

            ast::ASTNode::BinaryOp { op, left, right } => {
                let left_var = self.new_var();
                let right_var = self.new_var();

                let value = anf::ANFSimpleExpression::BinaryOp {
                    op: anf::ANFBinaryOp(op.clone()),
                    left: left_var,
                    right: right_var,
                };

                let result = anf::ANFNode::Let {
                    binding,
                    value,
                    body: Box::new(body),
                };

                let result = self.convert_bind_into(context.clone(), right_var, &right, result)?;
                self.convert_bind_into(context, left_var, &left, result)
            }

            ast::ASTNode::SpecialFunction(special) => {
                let value = anf::ANFSimpleExpression::SpecialFunction(special.clone());
                Ok(anf::ANFNode::Let {
                    binding,
                    value,
                    body: Box::new(body),
                })
            }
        };

        #[cfg(feature = "debug-ast-to-anf")]
        eprintln!(
            "convert_bind_into({:?}, {:?}, {:#?}, {:?}) = {:#?}",
            context_backup, binding, value, body_backup, result
        );
        result
    }

    fn convert_ast(&mut self, ast: ast::ASTNode) -> anyhow::Result<anf::ANFNode> {
        let result_var = self.new_var();
        self.convert_bind_into(
            im::HashMap::new(),
            result_var,
            &ast,
            anf::ANFNode::Simple(anf::ANFSimpleExpression::Var(result_var)),
        )
    }
}

impl TryFrom<ast::ASTNode> for anf::ANFNode {
    type Error = anyhow::Error;

    fn try_from(value: ast::ASTNode) -> Result<Self, Self::Error> {
        ASTToANFConverter::new().convert_ast(value)
    }
}
