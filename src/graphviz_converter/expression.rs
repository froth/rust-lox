use graphviz_rust::{dot_generator::*, dot_structures::*};

use crate::{
    ast::{
        expr::{Expr, ExprType},
        name::Name,
        token::Token,
    },
    graphviz_converter::random_id,
};

use super::{GraphvizConverter, GraphvizRepr};

impl GraphvizConverter for Expr {
    fn to_graphviz(&self) -> GraphvizRepr {
        self.expr_type.to_graphviz()
    }
}

impl GraphvizConverter for ExprType {
    fn to_graphviz(&self) -> GraphvizRepr {
        match self {
            ExprType::Assign(name, expr) => {
                single_child(format!("Assignment to \"{}\"", name.name).as_str(), expr)
            }
            ExprType::Binary(left, token, right) => binary(token, left, right),
            ExprType::Logical(left, token, right) => binary(token, left, right),
            ExprType::Grouping(expr) => single_child("Gouping", expr),
            ExprType::Literal(literal) => {
                GraphvizRepr::single(expr(format!("Literal: {}", literal).as_str()))
            }
            ExprType::Unary(operator, expr) => {
                single_child(format!("Unary {}", operator.token_type).as_str(), expr)
            }
            ExprType::Variable(name_expr) => {
                GraphvizRepr::single(expr(format!("Variable: {}", name_expr.name).as_str()))
            }
            ExprType::Call(callee, arguments) => call(callee, arguments),
            ExprType::Get(expr, name) => {
                single_child(format!("Get expression \"{}\"", name.name).as_str(), expr)
            }
            ExprType::Set(object, name, value) => set(object, &name.name, value),
            ExprType::This => GraphvizRepr::single(expr("this")),
            ExprType::Super(name) => {
                GraphvizRepr::single(expr(format!("super.{}", name.name).as_str()))
            }
        }
    }
}

fn expr(label: &str) -> Node {
    node!(esc random_id(); attr!("label", esc label.replace('\"', "\\\"")))
}

fn single_child(label: &str, expression: &Expr) -> GraphvizRepr {
    let mut node = GraphvizRepr::single(expr(label));
    let expr_repr = expression.to_graphviz();
    node.stmts.extend(expr_repr.stmts);
    node.push(edge!(node.id.clone() => expr_repr.id.clone()));
    node
}

fn binary(token: &Token, left: &Expr, right: &Expr) -> GraphvizRepr {
    let mut node = GraphvizRepr::single(expr(token.token_type.to_string().as_str()));
    let left = left.to_graphviz();
    node.stmts.extend(left.stmts);
    node.push(edge!(node.id.clone() => left.id.clone(); attr!("label", "left")));

    let right = right.to_graphviz();
    node.stmts.extend(right.stmts);
    node.push(edge!(node.id.clone() => right.id.clone(); attr!("label", "right")));
    node
}

fn call(callee: &Expr, arguments: &[Expr]) -> GraphvizRepr {
    let mut node = GraphvizRepr::single(expr("call"));
    let callee = callee.to_graphviz();
    node.stmts.extend(callee.stmts);
    node.push(edge!(node.id.clone() => callee.id.clone(); attr!("label", "callee")));

    let mut args = GraphvizRepr::single(expr("arguments"));
    arguments.iter().for_each(|a| {
        let a = a.to_graphviz();
        args.append(a.stmts);
        args.push(edge!(args.id.clone() => a.id))
    });

    node.stmts.extend(args.stmts);
    node.push(edge!(node.id.clone() => args.id.clone(); attr!("label", "arguments")));
    node
}

fn set(object: &Expr, name: &Name, value: &Expr) -> GraphvizRepr {
    let mut node = GraphvizRepr::single(expr(format!("set {}", name).as_str()));
    let object = object.to_graphviz();
    node.stmts.extend(object.stmts);
    node.push(edge!(node.id.clone() => object.id.clone(); attr!("label", "object")));

    let value = value.to_graphviz();
    node.stmts.extend(value.stmts);
    node.push(edge!(node.id.clone() => value.id.clone(); attr!("label", "value")));
    node
}
