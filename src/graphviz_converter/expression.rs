use graphviz_rust::{dot_generator::*, dot_structures::*};

use crate::{
    ast::expr::{Expr, ExprType},
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
            ExprType::Binary(_, _, _) => todo!(),
            ExprType::Logical(_, _, _) => todo!(),
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
            ExprType::Call(_, _) => todo!(),
            ExprType::Get(expr, name) => {
                single_child(format!("Get expression \"{}\"", name.name).as_str(), expr)
            }
            ExprType::Set(_, _, _) => todo!(),
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
    let mut expr_repr = expression.to_graphviz();
    node.stmts.append(&mut expr_repr.stmts);
    node.push(edge!(node.id.clone() => expr_repr.id.clone()));
    node
}
