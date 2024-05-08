use crate::ast::{
    self,
    expr::Expr,
    stmt::{self, Function, StmtType},
};
use graphviz_rust::{
    attributes::{rank, SubgraphAttributes},
    dot_generator::*,
    dot_structures::*,
};
use std::fmt::Write;

use super::{random_cluster_id, random_id, GraphvizConverter, GraphvizRepr};

impl GraphvizConverter for stmt::Stmt {
    fn to_graphviz(&self) -> GraphvizRepr {
        self.stmt_type.to_graphviz()
    }
}

impl GraphvizConverter for StmtType {
    fn to_graphviz(&self) -> GraphvizRepr {
        match self {
            StmtType::Expression(expr) => single_expr("Expr", expr),
            StmtType::Print(expr) => single_expr("print", expr),
            StmtType::Var { name, initializer } => todo!(),
            StmtType::Function(f) => function(f, "fun"),
            StmtType::Return(_) => todo!(),
            StmtType::Block(_) => todo!(),
            StmtType::If {
                condition,
                then_stmt,
                else_stmt,
            } => convert_if(condition, then_stmt, else_stmt.as_deref()),
            StmtType::While { condition, body } => todo!(),
            StmtType::Class {
                name,
                methods,
                superclass,
            } => todo!(),
        }
    }
}

fn single_expr(label: &str, expr: &Expr) -> GraphvizRepr {
    let mut print = GraphvizRepr::single(stmt(label));
    let expr = expr.to_graphviz();
    print.append(expr.stmts);
    print.push(edge!(print.id.clone() => expr.id));
    print
}

fn function(function: &Function, function_type: &str) -> GraphvizRepr {
    let name = &function.name;
    let mut parameters = String::new();
    function
        .parameters
        .iter()
        .for_each(|arg| write!(&mut parameters, "{arg}, ").unwrap());
    let parameters = parameters.trim_end_matches(", ");
    let label = format!("{function_type} {name}({parameters})");
    let mut node = GraphvizRepr::single(stmt(label.as_str()));
    let node_id = node.id.clone();
    let (body_ids, body_stmts): (Vec<_>, Vec<_>) = function
        .body
        .iter()
        .map(|s| s.to_graphviz())
        .map(|g| (g.id, g.stmts))
        .unzip();
    let mut body_stmts: Vec<Stmt> = body_stmts.into_iter().flatten().collect();
    let mut body_subgraph = subgraph!(esc random_cluster_id());
    body_subgraph.stmts.append(&mut body_stmts);
    body_ids.windows(2).for_each(|window| {
        body_subgraph
            .stmts
            .push(edge!(window[0].clone() => window[1].clone(); attr!("style", "invis")).into())
    });
    body_subgraph.stmts.push(attr!("style", "dotted").into());
    body_subgraph.stmts.push(attr!("label", "body").into());
    let mut rank_subgraph = subgraph!(esc random_id());
    body_ids.iter().for_each(|id| {
        rank_subgraph
            .stmts
            .push(Node::new(id.clone(), vec![]).into())
    });
    rank_subgraph.stmts.push(attr!("rank", "min").into());
    body_subgraph.stmts.push(rank_subgraph.into());
    node.push(body_subgraph);
    body_ids.into_iter().for_each(|id| {
        node.push(edge!(node_id.clone() => id; attr!("style", "dashed"), attr!("weight","0.9")))
    });
    node
}

fn convert_if(
    condition: &Expr,
    then_stmt: &stmt::Stmt,
    else_stmt: Option<&stmt::Stmt>,
) -> GraphvizRepr {
    let mut node = GraphvizRepr::single(stmt("if"));
    let condition = condition.to_graphviz();
    node.append(condition.stmts);
    node.push(edge!(node.id.clone() => condition.id; attr!("label", "condition")));
    node
}

fn stmt(label: &str) -> Node {
    node!(esc random_id(); attr!("shape", "rectangle"), attr!("label", esc label))
}
