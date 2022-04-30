use std::{collections::HashMap, env::VarError};

type VariableIndex = usize;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
enum BDDFunc {
    C(bool),
    N(usize), // Index into node-list
}

#[derive(Hash, PartialEq, Eq, Clone)]
struct BDDNode {
    label: VariableIndex,
    then_edge: BDDFunc,
    else_edge: BDDFunc,
}

struct BDD {
    nodes: Vec<BDDNode>,
    unique_table: HashMap<BDDNode, BDDFunc>,
}

fn top_variable(f: VariableIndex, g: BDDFunc, h: BDDFunc) -> VariableIndex {
    match (g, h) {
        (BDDFunc::C(_), BDDFunc::C(_)) => f,
        (BDDFunc::C(_), BDDFunc::N(h)) => std::cmp::min(f, h),
        (BDDFunc::N(g), BDDFunc::C(_)) => std::cmp::min(f, g),
        (BDDFunc::N(g), BDDFunc::N(h)) => std::cmp::min(f, std::cmp::min(g, h)),
    }
}

impl BDD {
    fn find_or_add_unique_table(&mut self, v: VariableIndex, t: BDDFunc, e: BDDFunc) -> BDDFunc {
        // TODO: t==e? why here?
        let new_node = BDDNode {
            label: v,
            then_edge: t,
            else_edge: e,
        };
        match self.unique_table.entry(new_node.clone()) {
            std::collections::hash_map::Entry::Occupied(o) => *o.get(),
            std::collections::hash_map::Entry::Vacant(v) => {
                self.nodes.push(new_node);
                let res = BDDFunc::N(self.nodes.len() - 1);
                v.insert(res);
                res
            }
        }
    }

    fn restrict(&mut self, f: BDDFunc, v: VariableIndex, value: bool) -> BDDFunc {
        match f {
            BDDFunc::C(_) => f,
            BDDFunc::N(n) => {
                let node = self.nodes[n].clone();
                if node.label == v {
                    if value {
                        node.then_edge
                    } else {
                        node.else_edge
                    }
                } else if v < node.label {
                    f
                } else {
                    // restrict below:
                    let then_node = self.restrict(node.then_edge, v, value);
                    let else_node = self.restrict(node.else_edge, v, value);
                    self.find_or_add_unique_table(v, then_node, else_node)
                }
            }
        }
    }

    fn ite(&mut self, f: BDDFunc, g: BDDFunc, h: BDDFunc) -> BDDFunc {
        // difference to paper: omitted computed-table
        match f {
            BDDFunc::C(true) => g,
            BDDFunc::C(false) => h,
            BDDFunc::N(f) => match (g, h) {
                (BDDFunc::C(true), BDDFunc::C(false)) => BDDFunc::N(f), // TODO; inverted case?
                _ => {
                    let v = top_variable(f, g, h);
                    let Fv = self.restrict(BDDFunc::N(f), v, true);
                    let Fnv = self.restrict(BDDFunc::N(f), v, false);
                    let Gv = self.restrict(g, v, true);
                    let Gnv = self.restrict(g, v, false);
                    let Hv = self.restrict(h, v, true);
                    let Hnv = self.restrict(h, v, false);
                    let T = self.ite(Fv, Gv, Hv);
                    let E = self.ite(Fnv, Gnv, Hnv);
                    // handle t==e?
                    self.find_or_add_unique_table(v, T, E)
                }
            },
        }
    }
}

fn main() {
    println!("Hello, world!");
}
