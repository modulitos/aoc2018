#[macro_use]
extern crate lazy_static;
use std::convert::TryFrom;
use std::str::FromStr;

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};

type Error = std::boxed::Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let graph = Graph::parse(&input)?;

    writeln!(
        std::io::stdout(),
        "topological sort: {}",
        graph.topo_sort().into_iter().collect::<String>()
    )?;

    Ok(())
}

type NodeId = char;

struct Graph {
    // Adjacency lists:
    incoming_list: HashMap<NodeId, HashSet<NodeId>>,
    outgoing_list: HashMap<NodeId, HashSet<NodeId>>,

    nodes: HashSet<NodeId>,
}

impl Graph {
    // returns a topological sort of the nodes
    fn topo_sort(&self) -> Vec<NodeId> {
        let mut visited = HashSet::<NodeId>::new();
        let mut sorted = Vec::<NodeId>::new();

        loop {
            if sorted.len() == self.nodes.len() {
                return sorted;
            }
            if let Some(&node_id) = self
                .nodes
                .iter()
                .filter(|&node_id| {
                    let has_deps = if let Some(incoming_nodes) = self.incoming_list.get(node_id) {
                        !incoming_nodes.is_subset(&visited)
                    } else {
                        // There are no nodes pointing to this node:
                        false
                    };
                    !has_deps && !sorted.contains(&node_id)
                })
                .min()
            {
                visited.insert(node_id);
                sorted.push(node_id);
            } else {
                panic!("Unable to find next node to visit - possible cycle detected");
            }
        }
    }

    fn parse(input: &str) -> Result<Self> {
        let edges = input
            .lines()
            .map(|line| line.parse())
            .collect::<Result<Vec<Edge>>>()?;

        let (incoming_list, outgoing_list, nodes) = edges.iter().fold(
            (
                HashMap::<NodeId, HashSet<NodeId>>::new(), // incoming
                HashMap::<NodeId, HashSet<NodeId>>::new(), // outgoing
                HashSet::<NodeId>::new(),
            ),
            |(mut incoming, mut outgoing, mut nodes), edge| {
                let Edge(from, to) = edge;
                incoming.entry(*to).or_default().insert(*from);
                outgoing.entry(*from).or_default().insert(*to);
                nodes.insert(*from);
                nodes.insert(*to);
                (incoming, outgoing, nodes)
            },
        );
        Ok(Graph {
            incoming_list,
            outgoing_list,
            nodes,
        })
    }
}

struct Edge(NodeId, NodeId);

impl FromStr for Edge {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"Step (?P<from>[A-Z]) must be finished before step (?P<to>[A-Z]) can begin."
            )
            .unwrap();
        }

        let caps = RE.captures(s).unwrap();
        // Note: we can shorten this like so:
        // let from = char::try_from(caps["from"].chars().next()?)?;
        // but only on nightly b/c NoneError is experimental: https://doc.rust-lang.org/std/option/struct.NoneError.html
        let from = char::try_from(caps["from"].chars().next().unwrap())?;
        let to = char::try_from(caps["to"].chars().next().unwrap())?;
        Ok(Edge(from, to))
    }
}

#[test]
fn test_get_steps() -> Result<()> {
    let s = "\
        Step C must be finished before step A can begin.\n\
        Step C must be finished before step F can begin.\n\
        Step A must be finished before step B can begin.\n\
        Step A must be finished before step D can begin.\n\
        Step B must be finished before step E can begin.\n\
        Step D must be finished before step E can begin.\n\
        Step F must be finished before step E can begin.\
    ";
    let graph = Graph::parse(&s)?;
    assert_eq!(graph.topo_sort(), vec!('C', 'A', 'B', 'D', 'F', 'E'));
    Ok(())
}

#[test]
fn test_subset() -> Result<()> {
    let sub = HashSet::<u32>::new();
    let sup = HashSet::<u32>::new();
    assert_eq!(sub.is_subset(&sup), true);
    println!("subset test passed!");
    Ok(())
}
