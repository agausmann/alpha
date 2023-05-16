use chumsky::{prelude::*, text};

fn tokenize() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let token = choice((
        text::keyword("fn").to(Token::Fn),
        text::keyword("struct").to(Token::Struct),
        just("->").to(Token::RightArrow),
        just("+").to(Token::Plus),
        just("-").to(Token::Minus),
        just(".").to(Token::Dot),
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

    let expr = recursive(|expr| {
        let field_init = ident
            .then_ignore(just(Token::Colon))
            .then(expr)
            .map(|(name, value)| FieldInit { name, value });

        let struct_init = ident
            .then_ignore(just(Token::OpenBracket))
            .then(field_init.separated_by(just(Token::Comma)).allow_trailing())
            .then_ignore(just(Token::CloseBracket))
            .map(|(name, fields)| StructInit { name, fields });

        let factor = choice((
            struct_init.map(Factor::StructInit),
            ident.map(Factor::Ident),
            int_literal.map(Factor::IntLiteral),
        ));
        let suffix_op = just(Token::Dot)
            .ignore_then(ident)
            .map(|field_name| SuffixOp::FieldAccess(field_name));
        // TODO prefix ops
        let term = factor
            .then(suffix_op.repeated())
            .map(|(factor, suffixes)| Term {
                prefixes: vec![],
                factor,
                suffixes,
            });

        let binary_op = select! {
            Token::Plus => BinaryOp::Plus,
            Token::Minus => BinaryOp::Minus,
        };

        term.clone()
            .then((binary_op.then(term)).repeated())
            .map(|(head, tail)| Expr { head, tail })
    });

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

    let field =
        ident
            .then_ignore(just(Token::Colon))
            .then(type_name)
            .map(|(field_name, field_type)| Field {
                field_name,
                field_type,
            });

    let struct_item = just(Token::Struct)
        .ignore_then(ident)
        .then_ignore(just(Token::OpenBracket))
        .then(field.separated_by(just(Token::Comma)).allow_trailing())
        .then_ignore(just(Token::CloseBracket))
        .map(|(name, fields)| StructItem { name, fields });

    let item = choice((fn_item.map(Item::Fn), struct_item.map(Item::Struct)));
    item.repeated()
        .then_ignore(end())
        .map(|items| Module { items })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Token {
    Ident(Ident),
    IntLiteral(IntLiteral),
    Fn,
    Struct,
    RightArrow,
    Plus,
    Minus,
    Colon,
    Dot,
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
    Struct(StructItem),
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
    prefixes: Vec<PrefixOp>,
    factor: Factor,
    suffixes: Vec<SuffixOp>,
}

#[derive(Debug, Clone)]
enum Factor {
    Ident(Ident),
    IntLiteral(IntLiteral),
    StructInit(StructInit),
}

#[derive(Debug, Clone)]
struct StructInit {
    name: Ident,
    fields: Vec<FieldInit>,
}

#[derive(Debug, Clone)]
struct FieldInit {
    name: Ident,
    value: Expr,
}

#[derive(Debug, Clone)]
enum PrefixOp {}

#[derive(Debug, Clone)]
enum SuffixOp {
    FieldAccess(Ident),
}

#[derive(Debug, Clone)]
enum BinaryOp {
    Plus,
    Minus,
}

#[derive(Debug, Clone)]
struct StructItem {
    name: Ident,
    fields: Vec<Field>,
}

#[derive(Debug, Clone)]
struct Field {
    field_name: Ident,
    field_type: Type,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_module(source: &str) {
        let tokens = tokenize().parse(source).unwrap();
        module().parse(tokens).unwrap();
    }

    #[test]
    fn add_two() {
        parse_module(include_str!("examples/add_two.pika"));
    }

    #[test]
    fn bijele() {
        parse_module(include_str!("examples/kattis/bijele.pika"));
    }
}
