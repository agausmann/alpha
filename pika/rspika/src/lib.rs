use chumsky::{prelude::*, text};

fn tokenize() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let token = choice((
        choice([
            text::keyword("break").to(Token::Break),
            text::keyword("else").to(Token::Else),
            text::keyword("enum").to(Token::Enum),
            text::keyword("fn").to(Token::Fn),
            text::keyword("for").to(Token::For),
            text::keyword("if").to(Token::If),
            text::keyword("in").to(Token::In),
            text::keyword("let").to(Token::Let),
            text::keyword("mut").to(Token::Mut),
            text::keyword("return").to(Token::Return),
            text::keyword("struct").to(Token::Struct),
        ]),
        choice([
            just("->").to(Token::RightArrow),
            just("..").to(Token::Dot2),
            just("::").to(Token::Colon2),
            just("==").to(Token::Eq2),
            just("&&").to(Token::And2),
            just("+").to(Token::Plus),
            just("-").to(Token::Minus),
            just(".").to(Token::Dot),
            just(",").to(Token::Comma),
            just(":").to(Token::Colon),
            just(";").to(Token::Semicolon),
            just("=").to(Token::Eq),
            just("!").to(Token::Exclam),
            just("(").to(Token::OpenParen),
            just(")").to(Token::CloseParen),
            just("[").to(Token::OpenBracket),
            just("]").to(Token::CloseBracket),
            just("{").to(Token::OpenBrace),
            just("}").to(Token::CloseBrace),
        ]),
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

    let path = ident
        .separated_by(just(Token::Colon2))
        .at_least(1)
        .map(|elements| Path { elements });

    let expr = recursive(|expr| {
        let field_init = ident
            .then_ignore(just(Token::Colon))
            .then(expr.clone())
            .map(|(name, value)| FieldInit { name, value });

        let struct_init = path
            .clone()
            .then_ignore(just(Token::OpenBrace))
            .then(field_init.separated_by(just(Token::Comma)).allow_trailing())
            .then_ignore(just(Token::CloseBrace))
            .map(|(name, fields)| StructInit { name, fields });

        let array_init = just(Token::OpenBracket)
            .ignore_then(choice((
                expr.clone()
                    .then_ignore(just(Token::Semicolon))
                    .then(int_literal)
                    .map(|(element, size)| ArrayInit::Fill {
                        element: Box::new(element),
                        size,
                    }),
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .map(ArrayInit::Elements),
            )))
            .then_ignore(just(Token::CloseBracket));

        let factor = choice((
            struct_init.map(Factor::StructInit),
            array_init.map(Factor::ArrayInit),
            path.clone().map(Factor::Path),
            int_literal.map(Factor::IntLiteral),
        ));
        let prefix_op = just(Token::Exclam).to(PrefixOp::Not);
        let suffix_op = choice((
            just(Token::Dot)
                .ignore_then(ident)
                .map(SuffixOp::FieldAccess),
            just(Token::OpenBracket)
                .ignore_then(expr.clone())
                .then_ignore(just(Token::CloseBracket))
                .map(SuffixOp::ArrayIndex),
        ));
        let term = prefix_op
            .repeated()
            .then(factor)
            .then(suffix_op.repeated())
            .map(|((prefixes, factor), suffixes)| Term {
                prefixes,
                factor,
                suffixes,
            });

        let binary_op = select! {
            Token::Plus => BinaryOp::Plus,
            Token::Minus => BinaryOp::Minus,
            Token::And2 => BinaryOp::LogicAnd,
            Token::Eq2 => BinaryOp::CmpEq,
        };

        term.clone()
            .then((binary_op.then(term)).repeated())
            .map(|(head, tail)| Expr { head, tail })
    });

    let type_name = recursive(|type_name| {
        let array_type = just(Token::OpenBracket)
            .ignore_then(type_name)
            .then_ignore(just(Token::Semicolon))
            .then(int_literal)
            .then_ignore(just(Token::CloseBracket))
            .map(|(element, size)| ArrayType {
                element: Box::new(element),
                size,
            });
        choice((path.map(Type::Path), array_type.map(Type::Array)))
    });

    let fn_arg = ident
        .then_ignore(just(Token::Colon))
        .then(type_name.clone())
        .map(|(arg_name, arg_type)| FnArg { arg_name, arg_type });

    let block = recursive(|block| {
        let let_statement = just(Token::Let)
            .ignore_then(just(Token::Mut).or_not())
            .then(ident)
            .then(just(Token::Colon).ignore_then(type_name.clone()).or_not())
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(((is_mut, binding), binding_type), value)| Let {
                is_mut: is_mut.is_some(),
                binding,
                binding_type,
                value,
            });
        let assign_statement = expr
            .clone()
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(dest, src)| Assign { dest, src });

        let if_statement = just(Token::If)
            .ignore_then(expr.clone())
            .then(block.clone())
            .map(|(condition, body)| IfCase { condition, body })
            .separated_by(just(Token::Else))
            .at_least(1)
            .then(just(Token::Else).ignore_then(block.clone()).or_not())
            .map(|(cases, else_case)| If { cases, else_case });

        let for_target = int_literal
            .then_ignore(just(Token::Dot2))
            .then(int_literal)
            .map(|(start, end)| ForTarget::Range(start, end));
        let for_statement = just(Token::For)
            .ignore_then(ident)
            .then_ignore(just(Token::In))
            .then(for_target)
            .then(block.clone())
            .map(|((binding, target), body)| For {
                binding,
                target,
                body,
            });
        let return_statement = just(Token::Return)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Semicolon));
        let break_statement = just(Token::Break).then(just(Token::Semicolon));
        let statement = choice((
            block.map(Statement::Block),
            let_statement.map(Statement::Let),
            assign_statement.map(Statement::Assign),
            if_statement.map(Statement::If),
            for_statement.map(Statement::For),
            return_statement.map(Statement::Return),
            break_statement.to(Statement::Break),
        ));

        just(Token::OpenBrace)
            .ignore_then(statement.repeated())
            .then(expr.or_not())
            .then_ignore(just(Token::CloseBrace))
            .map(|(statements, expr)| Block { statements, expr })
    });

    let fn_item = just(Token::Fn)
        .ignore_then(ident)
        .then(
            just(Token::OpenParen)
                .ignore_then(fn_arg.separated_by(just(Token::Comma)).allow_trailing())
                .then_ignore(just(Token::CloseParen)),
        )
        .then_ignore(just(Token::RightArrow))
        .then(type_name.clone())
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
        .then_ignore(just(Token::OpenBrace))
        .then(field.separated_by(just(Token::Comma)).allow_trailing())
        .then_ignore(just(Token::CloseBrace))
        .map(|(name, fields)| StructItem { name, fields });

    let variant = ident.map(|name| Variant { name });
    let enum_item = just(Token::Enum)
        .ignore_then(ident)
        .then_ignore(just(Token::OpenBrace))
        .then(variant.separated_by(just(Token::Comma)).allow_trailing())
        .then_ignore(just(Token::CloseBrace))
        .map(|(name, variants)| EnumItem { name, variants });

    let item = choice((
        fn_item.map(Item::Fn),
        struct_item.map(Item::Struct),
        enum_item.map(Item::Enum),
    ));
    item.repeated()
        .then_ignore(end())
        .map(|items| Module { items })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Token {
    Ident(Ident),
    IntLiteral(IntLiteral),
    Break,
    Else,
    Enum,
    Fn,
    For,
    If,
    In,
    Let,
    Mut,
    Return,
    Struct,
    RightArrow,
    Plus,
    Minus,
    Colon,
    Colon2,
    Semicolon,
    Dot,
    Dot2,
    Comma,
    Eq,
    Eq2,
    And2,
    Exclam,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Ident(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IntLiteral(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Path {
    elements: Vec<Ident>,
}

#[derive(Debug, Clone)]
struct Module {
    items: Vec<Item>,
}

#[derive(Debug, Clone)]
enum Item {
    Fn(FnItem),
    Struct(StructItem),
    Enum(EnumItem),
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
    Path(Path),
    Array(ArrayType),
}

#[derive(Debug, Clone)]
struct ArrayType {
    element: Box<Type>,
    size: IntLiteral,
}

#[derive(Debug, Clone)]
struct Block {
    statements: Vec<Statement>,
    expr: Option<Expr>,
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
    Path(Path),
    IntLiteral(IntLiteral),
    StructInit(StructInit),
    ArrayInit(ArrayInit),
}

#[derive(Debug, Clone)]
struct StructInit {
    name: Path,
    fields: Vec<FieldInit>,
}

#[derive(Debug, Clone)]
struct FieldInit {
    name: Ident,
    value: Expr,
}

#[derive(Debug, Clone)]
enum ArrayInit {
    Elements(Vec<Expr>),
    Fill {
        element: Box<Expr>,
        size: IntLiteral,
    },
}

#[derive(Debug, Clone)]
enum PrefixOp {
    Not,
}

#[derive(Debug, Clone)]
enum SuffixOp {
    FieldAccess(Ident),
    ArrayIndex(Expr),
}

#[derive(Debug, Clone)]
enum BinaryOp {
    Plus,
    Minus,
    CmpEq,
    LogicAnd,
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

#[derive(Debug, Clone)]
struct EnumItem {
    name: Ident,
    variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
struct Variant {
    name: Ident,
}

#[derive(Debug, Clone)]
enum Statement {
    Block(Block),
    Let(Let),
    Assign(Assign),
    If(If),
    For(For),
    Return(Expr),
    Break,
}

#[derive(Debug, Clone)]
struct Let {
    is_mut: bool,
    binding: Ident,
    binding_type: Option<Type>,
    value: Expr,
}

#[derive(Debug, Clone)]
struct Assign {
    dest: Expr,
    src: Expr,
}

#[derive(Debug, Clone)]
struct If {
    cases: Vec<IfCase>,
    else_case: Option<Block>,
}

#[derive(Debug, Clone)]
struct IfCase {
    condition: Expr,
    body: Block,
}

#[derive(Debug, Clone)]
struct For {
    binding: Ident,
    target: ForTarget,
    body: Block,
}

#[derive(Debug, Clone)]
enum ForTarget {
    Range(IntLiteral, IntLiteral),
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

    #[test]
    fn bluetooth() {
        parse_module(include_str!("examples/kattis/bluetooth.pika"));
    }
}
