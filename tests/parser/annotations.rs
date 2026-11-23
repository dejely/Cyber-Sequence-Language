use crate::expr::Expr;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::stmt::Stmt;
use crate::token::{Literal, TokenType};

fn program(source: &str) -> Vec<Stmt> {
    Parser::new(Scanner::new(source.to_string()).scan_tokens().unwrap())
        .parse_program()
        .unwrap()
        .statements
}

#[test]
fn annotation_tokens_keep_names_arguments_and_source_lines() {
    let tokens = Scanner::new("// @ignored\n@deprecated(\"old\")\nvar fruit;".into())
        .scan_tokens()
        .unwrap();
    assert_eq!(tokens[0].token_type(), &TokenType::At);
    assert_eq!(tokens[0].line(), 2);
    assert_eq!(tokens[1].lexeme(), "deprecated");
    assert_eq!(tokens[1].token_type(), &TokenType::Identifier);
    assert_eq!(tokens[3].literal(), Some(&Literal::String("old".into())));
    assert_eq!(tokens[5].token_type(), &TokenType::Var);
    assert_eq!(tokens[5].line(), 3);
}

#[test]
fn metadata_attaches_only_to_the_immediately_following_declaration() {
    let statements = program(concat!(
        "@deprecated\n// description\n@doc(\"Use freshFruit\") var oldFruit = \"banana\";\n",
        "var freshFruit = \"mango\";\n",
        "{ @doc(\"Local stock\") var stock = 3; var price; }",
    ));
    let Stmt::Var {
        annotations,
        name,
        initializer,
    } = &statements[0]
    else {
        panic!("expected variable declaration");
    };
    assert_eq!(name.lexeme(), "oldFruit");
    assert_eq!(
        *initializer,
        Some(Expr::Literal(Literal::String("banana".into())))
    );
    assert_eq!(annotations.len(), 2);
    assert_eq!(annotations[0].name.lexeme(), "deprecated");
    assert_eq!(annotations[0].name.line(), 1);
    assert!(annotations[0].arguments.is_empty());
    assert_eq!(annotations[1].name.lexeme(), "doc");
    assert_eq!(annotations[1].name.line(), 3);
    assert_eq!(
        annotations[1].arguments,
        vec![Literal::String("Use freshFruit".into())]
    );
    assert!(matches!(&statements[1], Stmt::Var { annotations, .. } if annotations.is_empty()));
    let Stmt::Block(locals) = &statements[2] else {
        panic!("expected block")
    };
    assert!(matches!(&locals[0], Stmt::Var { annotations, .. } if annotations.len() == 1));
    assert!(matches!(&locals[1], Stmt::Var { annotations, .. } if annotations.is_empty()));
}

#[test]
fn annotation_arguments_and_order_are_preserved() {
    let statements = program(
        "@doc(\"Old stock\") @deprecated(\"Use stock\") var oldStock; @deprecated() var oldPrice;",
    );
    let Stmt::Var { annotations, .. } = &statements[0] else {
        panic!("expected declaration")
    };
    assert_eq!(annotations[0].name.lexeme(), "doc");
    assert_eq!(
        annotations[1].arguments,
        vec![Literal::String("Use stock".into())]
    );
    assert!(
        matches!(&statements[1], Stmt::Var { annotations, .. } if annotations[0].arguments.is_empty())
    );

    let statements = program("for (@doc(\"Sale index\") var i = 0; i < 1; i = i + 1) {}");
    let Stmt::Block(desugared) = &statements[0] else {
        panic!("expected desugared for block")
    };
    let Stmt::Var {
        annotations, name, ..
    } = &desugared[0]
    else {
        panic!("expected loop initializer")
    };
    assert_eq!(name.lexeme(), "i");
    assert_eq!(
        annotations[0].arguments,
        vec![Literal::String("Sale index".into())]
    );
}

#[test]
fn recovery_retains_annotation_starters_and_block_boundaries() {
    let tokens = Scanner::new("{ var x = 1\n@doc() var y; }\n@deprecated print 2;".into())
        .scan_tokens()
        .unwrap();
    let error = Parser::new(tokens).parse_program().unwrap_err();
    assert_eq!(
        error,
        concat!(
            "[line 2] Error at '@': Expect ';' after variable declaration.\n",
            "[line 2] Error at 'doc': Annotation '@doc' requires one string argument.\n",
            "[line 3] Error at 'print': Annotations must precede a 'var' declaration."
        )
    );
}
