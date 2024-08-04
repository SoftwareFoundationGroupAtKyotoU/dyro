use dyro_poc::*;

fn integration_test_success(ast_node: ast::ASTNode, expected: interpreter::Value) {
    let anf: anf::ANFNode = ast_node.try_into().unwrap();
    let mut interpreter = interpreter::Interpreter::new();
    assert_eq!(interpreter.eval_node(&anf).unwrap(), expected);
}

fn integration_test_failure(ast_node: ast::ASTNode, expected_error: &str) {
    let anf: anf::ANFNode = ast_node.try_into().unwrap();
    let mut interpreter = interpreter::Interpreter::new();
    let result = interpreter.eval_node(&anf);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains(expected_error));
}

#[test]
fn let_test() {
    integration_test_success(
        dyro_poc::ast!(Let {
            binding: "x".into(),
            value: Box::new(Int(1)),
            body: Box::new(Var("x".into()))
        }),
        interpreter::Value::Int(1),
    );
}

#[test]
fn let_fn_test() {
    // let f x = x + 1 in f 2
    integration_test_success(
        dyro_poc::ast!(LetFn {
            name: "f".into(),
            args: vec![("x".into(), T::Int)],
            return_type: T::Int,
            value: Box::new(BinaryOp {
                op: Add,
                left: Box::new(Var("x".into())),
                right: Box::new(Int(1))
            }),
            body: Box::new(Call {
                function: Box::new(Var("f".into())),
                type_arguments: vec![],
                arguments: vec![Int(2)]
            })
        }),
        interpreter::Value::Int(3),
    );
}

#[test]
fn let_fn_wrong_type_test() {
    // let f x = x + 1 in f true
    integration_test_failure(
        dyro_poc::ast!(LetFn {
            name: "f".into(),
            args: vec![("x".into(), T::Int)],
            return_type: T::Int,
            value: Box::new(BinaryOp {
                op: Add,
                left: Box::new(Var("x".into())),
                right: Box::new(Int(1))
            }),
            body: Box::new(Call {
                function: Box::new(Var("f".into())),
                type_arguments: vec![],
                arguments: vec![Bool(true)]
            })
        }),
        "Invalid argument",
    );
}

#[test]
fn int_test() {
    integration_test_success(dyro_poc::ast!(Int(1)), interpreter::Value::Int(1));
}

#[test]
fn bool_test() {
    integration_test_success(dyro_poc::ast!(Bool(true)), interpreter::Value::Bool(true));
}

#[test]
fn string_test() {
    integration_test_success(
        dyro_poc::ast!(String("hello".into())),
        interpreter::Value::String("hello".into()),
    );
}

#[test]
fn if_test() {
    integration_test_success(
        dyro_poc::ast!(If {
            condition: Box::new(Bool(true)),
            then: Box::new(Int(1)),
            r#else: Box::new(Int(2)),
            r#type: T::Int,
        }),
        interpreter::Value::Int(1),
    );
}

#[test]
fn if_compare_test() {
    integration_test_success(
        dyro_poc::ast!(If {
            condition: Box::new(BinaryOp {
                op: Eq,
                left: Box::new(Int(1)),
                right: Box::new(Int(1))
            }),
            then: Box::new(Int(1)),
            r#else: Box::new(Int(2)),
            r#type: T::Int,
        }),
        interpreter::Value::Int(1),
    );
}

#[test]
fn addition_test() {
    integration_test_success(
        dyro_poc::ast!(BinaryOp {
            op: Add,
            left: Box::new(Int(1)),
            right: Box::new(Int(2))
        }),
        interpreter::Value::Int(3),
    );
}

#[test]
fn let_addition_test() {
    integration_test_success(
        dyro_poc::ast!(Let {
            binding: "x".into(),
            value: Box::new(Int(1)),
            body: Box::new(BinaryOp {
                op: Add,
                left: Box::new(Var("x".into())),
                right: Box::new(Int(2))
            })
        }),
        interpreter::Value::Int(3),
    );
}

#[test]
fn tuple_test() {
    integration_test_success(
        dyro_poc::ast!(Tuple {
            elements: vec![Int(1), Int(2), Int(3)]
        }),
        interpreter::Value::Tuple(vec![
            interpreter::Value::Int(1),
            interpreter::Value::Int(2),
            interpreter::Value::Int(3),
        ]),
    );
}

#[test]
fn tuple_access_test() {
    integration_test_success(
        dyro_poc::ast!(TupleAccess {
            tuple: Box::new(Tuple {
                elements: vec![Int(1), Int(2), Int(3)]
            }),
            index: 1
        }),
        interpreter::Value::Int(2),
    );
}

#[test]
fn panic_test() {
    // Call panic with message
    integration_test_failure(
        dyro_poc::ast!(Call {
            function: Box::new(SpecialFunction(crate::special::SpecialFunction::Panic)),
            type_arguments: vec![],
            arguments: vec![String("error".into())]
        }),
        "error",
    );
}

#[test]
fn alloc_poison_test() {
    // let x = alloc<int>(10) in x[0]
    integration_test_failure(
        dyro_poc::ast!(Let {
            binding: "x".into(),
            value: Box::new(Call {
                function: Box::new(SpecialFunction(crate::special::SpecialFunction::Alloc)),
                type_arguments: vec![T::Int],
                arguments: vec![Int(10)]
            }),
            body: Box::new(ArrayRead {
                array: Box::new(Var("x".into())),
                index: Box::new(Int(0))
            })
        }),
        "Poison",
    );
}

#[test]
fn alloc_read_write_test() {
    // let x = alloc<int>(10) in x[0] = 1; x[0]
    integration_test_success(
        dyro_poc::ast!(Let {
            binding: "x".into(),
            value: Box::new(Call {
                function: Box::new(SpecialFunction(crate::special::SpecialFunction::Alloc)),
                type_arguments: vec![T::Int],
                arguments: vec![Int(10)]
            }),
            body: Box::new(Sequence {
                first: Box::new(ArraySet {
                    array: Box::new(Var("x".into())),
                    index: Box::new(Int(0)),
                    value: Box::new(Int(1))
                }),
                second: Box::new(ArrayRead {
                    array: Box::new(Var("x".into())),
                    index: Box::new(Int(0))
                })
            })
        }),
        interpreter::Value::Int(1),
    );
}

#[test]
fn if_side_effect_test() {
    // if false then panic("error") else 1
    integration_test_success(
        dyro_poc::ast!(If {
            condition: Box::new(Bool(false)),
            then: Box::new(Call {
                function: Box::new(SpecialFunction(crate::special::SpecialFunction::Panic)),
                type_arguments: vec![],
                arguments: vec![String("error".into())]
            }),
            r#else: Box::new(Int(1)),
            r#type: T::Int,
        }),
        interpreter::Value::Int(1),
    );
}

#[test]
fn recursion_test() {
    // let pow2 x = if x == 0 then 1 else 2 * pow2 (x - 1) in pow2 3
    integration_test_success(
        dyro_poc::ast!(LetFn {
            name: "pow2".into(),
            args: vec![("x".into(), T::Int)],
            return_type: T::Int,
            value: Box::new(If {
                condition: Box::new(BinaryOp {
                    op: Eq,
                    left: Box::new(Var("x".into())),
                    right: Box::new(Int(0))
                }),
                then: Box::new(Int(1)),
                r#else: Box::new(BinaryOp {
                    op: Mul,
                    left: Box::new(Int(2)),
                    right: Box::new(Call {
                        function: Box::new(Var("pow2".into())),
                        type_arguments: vec![T::Int],
                        arguments: vec![BinaryOp {
                            op: Sub,
                            left: Box::new(Var("x".into())),
                            right: Box::new(Int(1))
                        }]
                    })
                }),
                r#type: T::Int,
            }),
            body: Box::new(Call {
                function: Box::new(Var("pow2".into())),
                type_arguments: vec![],
                arguments: vec![Int(3)]
            })
        }),
        interpreter::Value::Int(8),
    );
}

#[test]
fn copy_test() {
    /*
        let copy (newptr: *mut i32) (ptr: *mut i32) (len: i32) : Unit =
            let for (i: i32) : Unit =
                if i < len then
                    newptr[i] = ptr[i];
                    for (i + 1)
                else
                    ()
            in for 0
        in
        let x = alloc<int>(10) in
        let y = alloc<int>(10) in
        x[0] = 5;
        x[1] = 10;
        x[2] = 15;
        x[3] = 20;
        copy(y, x, 4);
        y[2]
    */
    integration_test_success(
        dyro_poc::ast!(LetFn {
            name: "copy".into(),
            args: vec![
                ("newptr".into(), T::MutPtr(Box::new(T::Int))),
                ("ptr".into(), T::MutPtr(Box::new(T::Int))),
                ("len".into(), T::Int),
            ],
            return_type: T::Unit,
            value: Box::new(LetFn {
                name: "for".into(),
                args: vec![("i".into(), T::Int)],
                return_type: T::Unit,
                value: Box::new(If {
                    condition: Box::new(BinaryOp {
                        op: Lt,
                        left: Box::new(Var("i".into())),
                        right: Box::new(Var("len".into())),
                    }),
                    then: Box::new(Sequence {
                        first: Box::new(ArraySet {
                            array: Box::new(Var("newptr".into())),
                            index: Box::new(Var("i".into())),
                            value: Box::new(ArrayRead {
                                array: Box::new(Var("ptr".into())),
                                index: Box::new(Var("i".into())),
                            }),
                        }),
                        second: Box::new(Call {
                            function: Box::new(Var("for".into())),
                            type_arguments: vec![],
                            arguments: vec![BinaryOp {
                                op: Add,
                                left: Box::new(Var("i".into())),
                                right: Box::new(Int(1)),
                            }],
                        }),
                    }),
                    r#else: Box::new(Unit),
                    r#type: T::Unit,
                }),
                body: Box::new(Call {
                    function: Box::new(Var("for".into())),
                    type_arguments: vec![],
                    arguments: vec![Int(0)],
                })
            }),
            body: Box::new(Let {
                binding: "x".into(),
                value: Box::new(Call {
                    function: Box::new(SpecialFunction(crate::special::SpecialFunction::Alloc)),
                    type_arguments: vec![T::Int],
                    arguments: vec![Int(10)],
                }),
                body: Box::new(Let {
                    binding: "y".into(),
                    value: Box::new(Call {
                        function: Box::new(SpecialFunction(crate::special::SpecialFunction::Alloc)),
                        type_arguments: vec![T::Int],
                        arguments: vec![Int(10)],
                    }),
                    body: Box::new(seq! {
                        ArraySet { array: Box::new(Var("x".into())), index: Box::new(Int(0)), value: Box::new(Int(5)) };
                        ArraySet { array: Box::new(Var("x".into())), index: Box::new(Int(1)), value: Box::new(Int(10)) };
                        ArraySet { array: Box::new(Var("x".into())), index: Box::new(Int(2)), value: Box::new(Int(15)) };
                        ArraySet { array: Box::new(Var("x".into())), index: Box::new(Int(3)), value: Box::new(Int(20)) };
                        Call {
                            function: Box::new(Var("copy".into())),
                            type_arguments: vec![],
                            arguments: vec![
                                Var("y".into()),
                                Var("x".into()),
                                Int(4),
                            ],
                        };
                        ArrayRead { array: Box::new(Var("y".into())), index: Box::new(Int(2)) }
                    })
                })
            })
        }),
        interpreter::Value::Int(15),
    );
}

#[test]
fn dealloc_test() {
    // let x = alloc<int>(10) in dealloc<int>(x, 10)
    integration_test_success(
        dyro_poc::ast!(Let {
            binding: "x".into(),
            value: Box::new(Call {
                function: Box::new(SpecialFunction(crate::special::SpecialFunction::Alloc)),
                type_arguments: vec![T::Int],
                arguments: vec![Int(10)],
            }),
            body: Box::new(Call {
                function: Box::new(SpecialFunction(crate::special::SpecialFunction::Dealloc)),
                type_arguments: vec![T::Int],
                arguments: vec![Var("x".into()), Int(10)],
            }),
        }),
        interpreter::Value::Unit,
    );
}

#[test]
fn dealloc_wrong_size_test() {
    // let x = alloc<int>(10) in dealloc<int>(x, 5)
    integration_test_failure(
        dyro_poc::ast!(Let {
            binding: "x".into(),
            value: Box::new(Call {
                function: Box::new(SpecialFunction(crate::special::SpecialFunction::Alloc)),
                type_arguments: vec![T::Int],
                arguments: vec![Int(10)],
            }),
            body: Box::new(Call {
                function: Box::new(SpecialFunction(crate::special::SpecialFunction::Dealloc)),
                type_arguments: vec![T::Int],
                arguments: vec![Var("x".into()), Int(5)],
            }),
        }),
        "Invalid size",
    );
}

#[test]
fn dealloc_different_type_test() {
    // let x = alloc<int>(10) in dealloc<bool>(x, 40)
    integration_test_success(
        dyro_poc::ast!(Let {
            binding: "x".into(),
            value: Box::new(Call {
                function: Box::new(SpecialFunction(crate::special::SpecialFunction::Alloc)),
                type_arguments: vec![T::Int],
                arguments: vec![Int(10)],
            }),
            body: Box::new(Call {
                function: Box::new(SpecialFunction(crate::special::SpecialFunction::Dealloc)),
                type_arguments: vec![T::Bool],
                arguments: vec![Var("x".into()), Int(40)],
            }),
        }),
        interpreter::Value::Unit,
    );
}

#[test]
fn read_after_dealloc_test() {
    // let x = alloc<int>(10) in (x[0] = 5; dealloc<int>(x, 10); x[0])
    integration_test_failure(
        dyro_poc::ast!(Let {
            binding: "x".into(),
            value: Box::new(Call {
                function: Box::new(SpecialFunction(crate::special::SpecialFunction::Alloc)),
                type_arguments: vec![T::Int],
                arguments: vec![Int(10)],
            }),
            body: Box::new(seq! {
                ArraySet { array: Box::new(Var("x".into())), index: Box::new(Int(0)), value: Box::new(Int(5)) };
                Call {
                    function: Box::new(SpecialFunction(crate::special::SpecialFunction::Dealloc)),
                    type_arguments: vec![T::Int],
                    arguments: vec![Var("x".into()), Int(10)],
                };
                ArrayRead { array: Box::new(Var("x".into())), index: Box::new(Int(0)) }
            }),
        }),
        "Undefined",
    );
}

#[test]
fn print_test() {
    // print("hello")
    integration_test_success(
        dyro_poc::ast!(Call {
            function: Box::new(SpecialFunction(crate::special::SpecialFunction::Print)),
            type_arguments: vec![T::String],
            arguments: vec![String("hello".into())],
        }),
        interpreter::Value::Unit,
    );
}

#[test]
fn print_bad_type_test() {
    // print(1)
    integration_test_failure(
        dyro_poc::ast!(Call {
            function: Box::new(SpecialFunction(crate::special::SpecialFunction::Print)),
            type_arguments: vec![T::String],
            arguments: vec![Int(1)],
        }),
        "Invalid type for Print",
    );
}
