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
        graph.iter_topo_sort().collect::<Result<String, String>>()?
    )?;

    let workers = WorkerPool::new(5);

    writeln!(
        std::io::stdout(),
        "time to process: {}",
        workers.run_simulation(&graph)
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
    // given a set of accessible nodes, returns a Vec of the next neighboring nodes
    fn next_accessible_nodes(&self, accessible_nodes: &HashSet<NodeId>) -> HashSet<NodeId> {
        self.nodes
            .iter()
            .filter_map(|&node_id| {
                let has_deps = if let Some(incoming_nodes) = self.incoming_list.get(&node_id) {
                    // All nodes pointing to this node have already been visited
                    !incoming_nodes.is_subset(&accessible_nodes)
                } else {
                    // There are no nodes pointing to this node:
                    false
                };
                if !has_deps && !accessible_nodes.contains(&node_id) {
                    Some(node_id)
                } else {
                    None
                }
            })
            .collect::<HashSet<NodeId>>()
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

    pub fn iter_topo_sort(&self) -> IterGraph {
        IterGraph {
            visited: HashSet::new(),
            graph: &self,
        }
    }
}

struct IterGraph<'a> {
    visited: HashSet<NodeId>,
    graph: &'a Graph,
}

// Iterates over nodes in a topological sorted order
impl<'a> Iterator for IterGraph<'a> {
    // Represents an ordered set of Nodes that have the same topological ordering
    type Item = Result<NodeId, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.visited.len() == self.graph.nodes.len() {
            return None;
        }
        let next_accessible = self.graph.next_accessible_nodes(&self.visited);
        if let Some(&next) = next_accessible.iter().min() {
            self.visited.insert(next);
            Some(Ok(next))
        } else {
            return Some(Err(String::from(
                "Unable to find next node to visit - possible cycle detected",
            )));
        }
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

        if !from.is_ascii_uppercase() || !to.is_ascii_uppercase() {
            return Err(Error::from(format!(
                "Node should be ascii uppercase: {}, {}",
                to, from
            )));
        }

        Ok(Edge(from, to))
    }
}

type Time = u32;
type WorkerId = usize;

#[derive(PartialEq, Debug)]
enum Status {
    Idle,
    Busy { until: Time, node: NodeId }, // busy until this time
}

struct WorkerPool {
    //    num_workers: u8,
    workers: Vec<Status>,
    is_simple: bool,
    time: Time,
    processed: HashSet<NodeId>,
    in_progress: HashSet<NodeId>,
}

impl WorkerPool {
    fn new(n: u8) -> Self {
        WorkerPool {
            workers: (0..n).map(|_| Status::Idle).collect::<Vec<Status>>(),
            is_simple: false,
            time: 0,
            processed: HashSet::<NodeId>::new(),
            in_progress: HashSet::<NodeId>::new(),
        }
    }

    fn simple(mut self) -> Self {
        self.is_simple = true;
        self
    }

    // 'A' -> 61 (or 1 if simple)
    // 'B' -> 62 (or 2 if simple)
    // Ascii for 'A' is 65

    fn get_node_duration(node: NodeId, is_simple: bool) -> Time {
        let ascii_value = u32::from(node);
        if is_simple {
            ascii_value - 64
        } else {
            ascii_value - 4
        }
    }

    // update our nodes that have finished processing

    fn update_processed_nodes(&mut self) {
        use Status::*;

        let current_time = self.time;

        // TODO: How to avoid "cannot move out of mutable reference" without having to move them
        // here?

        let mut processed = std::mem::replace(&mut self.processed, HashSet::new());
        let mut in_progress = std::mem::replace(&mut self.in_progress, HashSet::new());
        self.workers
            .iter_mut()
            .filter(|status| match status {
                Idle => false,
                Busy { until, .. } => until <= &current_time,
            })
            .for_each(|status| {
                if let Busy {
                    node: finished_node,
                    ..
                } = status
                {
                    processed.insert(*finished_node);
                    in_progress.remove(finished_node);
                    *status = Idle;
                } else {
                    panic!("invalid state - we should be filtering these out!")
                }
            });
        self.processed = processed;
        self.in_progress = in_progress;
    }

    // process the nodes until either they run out or all of the workers are busy.

    // If all of the workers are busy, advance the time until the shortest job is finished and exit.

    fn process_second(&mut self, mut nodes: Vec<NodeId>) {
        use Status::*;

        // Update any new nodes that will now be processed
        nodes.sort();

        let mut in_progress = std::mem::replace(&mut self.in_progress, HashSet::new());
        let updated_workers = self
            .workers
            .iter()
            .enumerate()
            .filter(|&(_worker_id, status)| status == &Idle)
            .zip(nodes.iter())
            .map(|((worker_id, _status), &node_id)| {
                in_progress.insert(node_id);
                let job_length = WorkerPool::get_node_duration(node_id, self.is_simple);

                (
                    worker_id,
                    Busy {
                        until: self.time + job_length,
                        node: node_id,
                    },
                )
            })
            .collect::<Vec<(WorkerId, Status)>>();
        self.in_progress = in_progress;

        updated_workers.into_iter().for_each(|(worker_id, status)| {
            self.workers[worker_id] = status;
        });
    }

    // Gets the time it takes to complete the graph in topological order, while delegating to
    // workers

    fn run_simulation(mut self, graph: &Graph) -> u32 {
        self.time = 0;
        loop {
            self.update_processed_nodes();
            let nodes_ready_for_workers = graph
                .next_accessible_nodes(&self.processed)
                .into_iter()
                // Omit nodes that are already in progress:
                .filter(|node_id| !self.in_progress.contains(node_id))
                .collect::<Vec<NodeId>>();

            self.process_second(nodes_ready_for_workers);
            if self.workers.iter().all(|status| status == &Status::Idle) {
                break;
            }
            self.time += 1;
        }
        self.time
    }
}

#[test]
fn test_topo_sort() -> Result<()> {
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
    assert_eq!(
        graph
            .iter_topo_sort()
            .collect::<Result<Vec<NodeId>, String>>()?,
        vec!('C', 'A', 'B', 'D', 'F', 'E')
    );
    println!("test_topo_sort passed");
    Ok(())
}

#[test]
fn test_completion_time() -> Result<()> {
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
    let mut workers = WorkerPool::new(2);
    workers = workers.simple();

    assert_eq!(workers.run_simulation(&graph), 15);
    // Non-simple:
    //    assert_eq!(workers.run_simulation(&graph), 258);
    println!("test_completion_time passed");
    Ok(())
}
