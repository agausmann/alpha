use chumsky::{prelude::*, text};

fn main() -> anyhow::Result<()> {
    let source = std::fs::read_to_string("../examples/add_two.pika")?;
    let tokens = match tokenize().parse(source) {
        Ok(x) => x,
        Err(errs) => {
            for err in errs {
                eprintln!("{:?}", err);
            }
            return Ok(());
        }
    };

    let module = match module().parse(tokens) {
        Ok(x) => x,
        Err(errs) => {
            for err in errs {
                eprintln!("{:?}", err);
            }
            return Ok(());
        }
    };

    println!("{:#?}", module);

    Ok(())
}

fn tokenize() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let token = choice((
        text::keyword("fn").to(Token::Fn),
        just("->").to(Token::RightArrow),
        just("+").to(Token::Plus),
        just(",").to(Token::Comma),
        just(":").to(Token::Colon),
        just("(").to(Token::OpenParen),
        just(")").to(Token::CloseParen),
        just("{").to(Token::OpenBracket),
        just("}").to(Token::CloseBracket),
        text::ident().map(|s| Token::Ident(Ident(s))),
        text::int(10).map(|s| Token::IntLiteral(IntLiteral(s))),
    ));

    let line_comment = just("//").then(take_until(text::newline())).ignored();
    let block_comment = just("/*").then(take_until(just("*/"))).ignored();

    let whitespace = choice((
        text::whitespace().at_least(1).ignored(),
        line_comment,
        block_comment,
    ))
    .repeated();

    (whitespace.ignore_then(token))
        .repeated()
        .then_ignore(whitespace)
        .then_ignore(end())
}

fn module() -> impl Parser<Token, Module, Error = Simple<Token>> {
    let ident = select! {
        Token::Ident(x) => x,
    };
    let int_literal = select! {
        Token::IntLiteral(x) => x,
    };

    let factor = choice((
        ident.map(Factor::Ident),
        int_literal.map(Factor::IntLiteral),
    ));
    // TODO unary ops
    let term = factor.map(|factor| Term {
        ops: vec![],
        factor,
    });

    let binary_op = select! {
        Token::Plus => BinaryOp::Plus,
    };

    let expr = term
        .then((binary_op.then(term)).repeated())
        .map(|(head, tail)| Expr { head, tail });

    let type_name = ident.map(Type::Ident);

    let fn_arg = ident
        .then_ignore(just(Token::Colon))
        .then(type_name)
        .map(|(arg_name, arg_type)| FnArg { arg_name, arg_type });

    let block = just(Token::OpenBracket)
        .ignore_then(expr)
        .then_ignore(just(Token::CloseBracket))
        .map(|expr| Block { expr });

    let fn_item = just(Token::Fn)
        .ignore_then(ident)
        .then(
            just(Token::OpenParen)
                .ignore_then(fn_arg.separated_by(just(Token::Comma)).allow_trailing())
                .then_ignore(just(Token::CloseParen)),
        )
        .then_ignore(just(Token::RightArrow))
        .then(type_name)
        .then(block)
        .map(|(((name, args), return_type), body)| FnItem {
            name,
            args,
            return_type,
            body,
        });
    let item = fn_item.map(Item::Fn);
    item.repeated()
        .then_ignore(end())
        .map(|items| Module { items })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Token {
    Ident(Ident),
    IntLiteral(IntLiteral),
    Fn,
    RightArrow,
    Plus,
    Colon,
    Comma,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Ident(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IntLiteral(String);

#[derive(Debug, Clone)]
struct Module {
    items: Vec<Item>,
}

#[derive(Debug, Clone)]
enum Item {
    Fn(FnItem),
}

#[derive(Debug, Clone)]
struct FnItem {
    name: Ident,
    args: Vec<FnArg>,
    return_type: Type,
    body: Block,
}

#[derive(Debug, Clone)]
struct FnArg {
    arg_name: Ident,
    arg_type: Type,
}

#[derive(Debug, Clone)]
enum Type {
    Ident(Ident),
}

#[derive(Debug, Clone)]
struct Block {
    expr: Expr,
}

#[derive(Debug, Clone)]
struct Expr {
    head: Term,
    tail: Vec<(BinaryOp, Term)>,
}

#[derive(Debug, Clone)]
struct Term {
    ops: Vec<UnaryOp>,
    factor: Factor,
}

#[derive(Debug, Clone)]
enum Factor {
    Ident(Ident),
    IntLiteral(IntLiteral),
}

#[derive(Debug, Clone)]
enum UnaryOp {}

#[derive(Debug, Clone)]
enum BinaryOp {
    Plus,
}
