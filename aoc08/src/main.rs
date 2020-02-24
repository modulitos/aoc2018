use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::num::ParseIntError;
use std::str::SplitAsciiWhitespace;
use std::iter::FromIterator;

type Error = std::boxed::Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let graph = Tree::parse(&input)?;

    writeln!(
        std::io::stdout(),
        "sum of metadata: {}",
        graph.sum_metadata(),
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
    ) -> Result<(Self, HashMap<NodeId, Self>, SplitAsciiWhitespace)> {
        if let (Some(children_str), Some(metadata_str)) = (iter.next(), iter.next()) {
            let (num_children, num_metadata) =
                (children_str.parse::<u32>()?, metadata_str.parse::<usize>()?);
            let curr_node_id = id;
            id += 1;
            let mut nodes = HashMap::<NodeId, Node>::new();
            let mut children = vec![];
            for _ in 0..num_children {
                let (child_node, mut new_nodes, next_iter) = Node::parse(iter, id)?;
                iter = next_iter; // re-assign the input for the next iteration
                id = id.saturating_add(children.len() as u16).saturating_add(1);
                children.push(child_node.id);
                nodes.insert(child_node.id, child_node);
                nodes.extend(new_nodes.into_iter());
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
    nodes: HashMap<NodeId, Node>,
    root: NodeId,
}

impl Tree {
    fn parse(input: &str) -> Result<Self> {
        let (root, mut nodes, ..) = Node::parse(input.split_ascii_whitespace(), 0)?;
        let root_id = root.id;
        nodes.insert(root.id, root);
        println!("nodes: {:?}", nodes);
        Ok(Tree {
            nodes,
            root: root_id,
        })
    }

    fn sum_metadata(&self) -> u32 {
        self.nodes
            .values()
            .flat_map(|node| node.metadata.clone())
            .sum::<u32>()
    }
}

#[test]
fn test_metadata_sum() -> Result<()> {
    let input = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";
    let tree = Tree::parse(&input)?;
    assert_eq!(
        tree.sum_metadata(),
        138
    );
    assert_eq!(
        tree.nodes.keys().collect::<HashSet<&NodeId>>(),
        HashSet::<&NodeId>::from_iter(vec![0, 1, 2, 3].iter())
    );
    assert_eq!(
        tree.nodes.get(&0).unwrap().children,
        vec![1, 2]
    );
    assert_eq!(
        tree.nodes.get(&2).unwrap().children,
        vec![3]
    );
    println!("test_metadata_sum passed.");
    Ok(())
}

fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

#[test]
fn test_sample_input_metadata() -> Result<()> {
    let file_name = PathBuf::from("./input/input.txt");
    println!("file_name: {:?}", file_name);
    // gets the file path relative to the cargo project dir
    let file_path = canonicalize(&file_name)?;
    println!("file_path: {:?}", file_path);
    let input = &lines_from_file(file_path)[0];
    let tree = Tree::parse(&input)?;
    assert_eq!(tree.sum_metadata(), 37905);
    println!("test_sample_metadata_sum passed.");
    Ok(())
}

// #[test]
fn test_root_node_value() -> Result<()> {
    let input = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";
    let tree = Tree::parse(&input)?;
    assert_eq!(tree.get_root_value(), 66);
    println!("test_root_node_value passed.");
    Ok(())
}

