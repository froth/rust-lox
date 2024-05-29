use crate::ast::{
    expr::Expr,
    name::{Name, NameExpr},
    stmt::{self, Function, StmtType},
};
use graphviz_rust::{dot_generator::*, dot_structures::*};
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
            StmtType::Var { name, initializer } => {
                single_with_option_expr(format!("var {}", name).as_str(), initializer)
            }
            StmtType::Function(f) => function(f, "fun"),
            StmtType::Return(expr) => single_with_option_expr("return", expr),
            StmtType::Block(stmts) => block(stmts, "block"),
            StmtType::If {
                condition,
                then_stmt,
                else_stmt,
            } => convert_if(condition, then_stmt, else_stmt.as_deref()),
            StmtType::While { condition, body } => convert_while(condition, body),
            StmtType::Class {
                name,
                methods,
                superclass,
            } => class(name, methods, superclass),
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

fn rank_subgraph(ids: &[NodeId]) -> (Subgraph, Vec<Stmt>) {
    let mut rank_subgraph = subgraph!(esc random_id());
    ids.iter().for_each(|id| {
        rank_subgraph
            .stmts
            .push(Node::new(id.clone(), vec![]).into())
    });
    rank_subgraph.stmts.push(attr!("rank", "same").into());
    let invisible_edges = ids
        .windows(2)
        .map(|window| edge!(window[0].clone() => window[1].clone(); attr!("style", "invis")).into())
        .collect();
    (rank_subgraph, invisible_edges)
}

fn block(block: &[stmt::Stmt], label: &str) -> GraphvizRepr {
    let mut node = GraphvizRepr::single(stmt(label));
    let node_id = node.id.clone();
    let (ids, stmts): (Vec<_>, Vec<_>) = block
        .iter()
        .map(|s| s.to_graphviz())
        .map(|g| (g.id, g.stmts))
        .unzip();
    let mut stmts: Vec<Stmt> = stmts.into_iter().flatten().collect();
    let mut subgraph = subgraph!(esc random_cluster_id());
    subgraph.stmts.append(&mut stmts);
    subgraph.stmts.push(attr!("style", "dotted").into());
    let (rank_subgraph, mut rank_edges) = rank_subgraph(&ids);
    subgraph.stmts.push(rank_subgraph.into());
    subgraph.stmts.append(&mut rank_edges);
    node.push(subgraph);
    ids.into_iter()
        .for_each(|id| node.push(edge!(node_id.clone() => id; attr!("style", "dashed"))));
    node
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
    block(&function.body, label.as_str())
}

fn convert_if(
    condition: &Expr,
    then_stmt: &stmt::Stmt,
    else_stmt: Option<&stmt::Stmt>,
) -> GraphvizRepr {
    let mut node = GraphvizRepr::single(stmt("if"));
    let mut ids = vec![];
    let condition = condition.to_graphviz();
    ids.push(condition.id.clone());
    node.append(condition.stmts);
    node.push(edge!(node.id.clone() => condition.id; attr!("label", "condition")));
    let then = then_stmt.to_graphviz();
    ids.push(then.id.clone());
    node.append(then.stmts);
    node.push(edge!(node.id.clone() => then.id; attr!("label", "then")));
    else_stmt.into_iter().for_each(|e| {
        let else_node = e.to_graphviz();
        ids.push(else_node.id.clone());
        node.append(else_node.stmts);
        node.push(edge!(node.id.clone() => else_node.id; attr!("label", "else")));
    });
    node
}

fn convert_while(condition: &Expr, body: &stmt::Stmt) -> GraphvizRepr {
    let mut node = GraphvizRepr::single(stmt("while"));
    let mut ids = vec![];
    let condition = condition.to_graphviz();
    let mut subgraph = subgraph!(esc random_id());
    ids.push(condition.id.clone());
    subgraph.stmts.extend(condition.stmts);
    let body = body.to_graphviz();
    ids.push(body.id.clone());
    subgraph.stmts.extend(body.stmts);
    let (rank_subgraph, mut rank_edges) = rank_subgraph(&ids);
    subgraph.stmts.push(rank_subgraph.into());
    node.push(edge!(node.id.clone() => condition.id; attr!("label", "condition")));
    node.push(edge!(node.id.clone() => body.id; attr!("label", "body")));
    node.stmts.push(subgraph.into());
    node.stmts.append(&mut rank_edges);
    node
}

fn single_with_option_expr(label: &str, expr: &Option<Expr>) -> GraphvizRepr {
    let mut node = GraphvizRepr::single(stmt(label));
    expr.iter().for_each(|e| {
        let e = e.to_graphviz();
        node.append(e.stmts);
        node.stmts.push(edge!(node.id.clone() => e.id).into());
    });
    node
}

fn class(name: &Name, methods: &[Function], superclass: &Option<NameExpr>) -> GraphvizRepr {
    let label = match superclass {
        Some(superclass) => format!("class {} < {}", name, superclass.name),
        None => format!("class {}", name),
    };
    let mut node = GraphvizRepr::single(stmt(label.as_str()));
    let node_id = node.id.clone();
    let (ids, stmts): (Vec<_>, Vec<_>) = methods
        .iter()
        .map(|f| function(f, "method"))
        .map(|g| (g.id, g.stmts))
        .unzip();
    let mut stmts: Vec<Stmt> = stmts.into_iter().flatten().collect();
    let mut subgraph = subgraph!(esc random_cluster_id());
    subgraph.stmts.append(&mut stmts);
    subgraph.stmts.push(attr!("style", "dotted").into());
    let (rank_subgraph, mut rank_edges) = rank_subgraph(&ids);
    subgraph.stmts.push(rank_subgraph.into());
    subgraph.stmts.append(&mut rank_edges);
    node.push(subgraph);
    ids.into_iter()
        .for_each(|id| node.push(edge!(node_id.clone() => id; attr!("style", "dashed"))));
    node
}

fn stmt(label: &str) -> Node {
    node!(esc random_id(); attr!("shape", "rectangle"), attr!("label", esc label))
}
