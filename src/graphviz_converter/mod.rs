mod expression;
mod statement;
use graphviz_rust::{
    dot_generator::*,
    dot_structures::{Stmt as GVStmt, *},
    printer::{DotPrinter, PrinterContext},
};
use uuid::Uuid;

use crate::ast::stmt::Stmt;

struct GraphvizRepr {
    stmts: Vec<GVStmt>,
    id: NodeId,
}

impl GraphvizRepr {
    fn single(node: Node) -> Self {
        let id = node.id.clone();
        Self {
            stmts: vec![node.into()],
            id,
        }
    }

    fn append(&mut self, mut stmts: Vec<GVStmt>) {
        self.stmts.append(&mut stmts);
    }

    fn push<A: Into<GVStmt>>(&mut self, value: A) {
        self.stmts.push(value.into())
    }
}

trait GraphvizConverter {
    fn to_graphviz(&self) -> GraphvizRepr;
}

fn random_id() -> String {
    Uuid::new_v4().to_string()
}

fn random_cluster_id() -> String {
    format!("cluster_{}", Uuid::new_v4())
}

pub fn print_graphviz(statements: Vec<Stmt>) {
    let graph = to_graphviz(statements);
    print!("{}", graph.print(&mut PrinterContext::default()));
}

fn to_graphviz(statements: Vec<Stmt>) -> Graph {
    let nodes = statements
        .iter()
        .map(|s| {
            let mut stmts = s.to_graphviz().stmts;
            stmts.push(attr!("style", "dotted").into());
            subgraph!(esc random_cluster_id(), stmts).into()
        })
        .collect();
    Graph::DiGraph {
        id: id!("id"),
        strict: true,
        stmts: nodes,
    }
}
