use std::io::{Read, Write};
use std::iter::Enumerate;
use std::num::ParseIntError;
use std::ops::Add;
use std::str::{SplitAsciiWhitespace, SplitWhitespace};

type Error = std::boxed::Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let graph = Tree::parse(&input)?;

    writeln!(
        std::io::stdout(),
        "sum of metadata: {}",
        graph
            .nodes
            .iter()
            .flat_map(|node| node.metadata.clone())
            .sum::<u32>()
    )?;

    Ok(())
}

type NodeId = u16;

#[derive(Debug)]
struct Node {
    id: NodeId,
    metadata: Vec<u32>,    // 1 or more
    children: Vec<NodeId>, // 0 or more
}

impl Node {
    // Is there a better option besides having to use an external iterator, transferring its
    // ownership, and having to return it?

    fn parse(
        mut iter: SplitAsciiWhitespace,
        mut id: NodeId,
    ) -> Result<(Self, Vec<Self>, SplitAsciiWhitespace)> {
        if let (Some(children_str), Some(metadata_str)) = (iter.next(), iter.next()) {
            let (num_children, num_metadata) =
                (children_str.parse::<u32>()?, metadata_str.parse::<usize>()?);
            let curr_node_id = id;
            id += 1;
            let mut nodes = Vec::<Node>::new();
            let mut children = vec![];
            for _ in 0..num_children {
                let (child_node, mut new_nodes, next_iter) = Node::parse(iter, id)?;
                iter = next_iter; // re-assign the input for the next iteration
                id = id.saturating_add(children.len() as u16).saturating_add(1);
                children.push(child_node.id);
                nodes.push(child_node);
                nodes.append(&mut new_nodes);
            }
            let node = Node {
                id: curr_node_id,
                metadata: iter
                    .by_ref()
                    .take(num_metadata)
                    .map(|metadata_string| metadata_string.parse::<u32>())
                    .collect::<Result<Vec<u32>, ParseIntError>>()?,
                children,
            };
            Ok((node, nodes, iter))
        } else {
            // TODO: make this an error instead of a panic
            panic!("Invalid iterator size")
        }
    }
}

struct Tree {
    // TODO: this should be a hashmap of NodeId to node
    nodes: Vec<Node>,
    root: NodeId,
}

impl Tree {
    fn parse(input: &str) -> Result<Self> {
        let (root, mut nodes, ..) = Node::parse(input.split_ascii_whitespace(), 0)?;
        let root_id = root.id;
        nodes.push(root);
        println!("nodes: {:?}", nodes);
        Ok(Tree {
            nodes,
            root: root_id,
        })
    }
}

#[test]
fn test_metadata_sum() -> Result<()> {
    let input = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";
    let graph = Tree::parse(&input)?;
    assert_eq!(
        graph
            .nodes
            .iter()
            .flat_map(|node| node.metadata.clone())
            .sum::<u32>(),
        138
    );
    println!("test_metadata_sum passed.");
    Ok(())
}
