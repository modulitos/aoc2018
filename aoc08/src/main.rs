use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::num::ParseIntError;
use std::str::SplitAsciiWhitespace;

use std::{
    fs::{canonicalize, File},
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
};

type Error = std::boxed::Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let tree = Tree::parse(&input)?;

    // writeln!(std::io::stdout(), "tree: {}", tree,)?;

    writeln!(
        std::io::stdout(),
        "sum of metadata: {}",
        tree.sum_metadata(),
    )?;

    writeln!(
        std::io::stdout(),
        "tree root value: {}",
        tree.get_root_value(),
    )?;

    Ok(())
}

type NodeId = u32;

#[derive(Debug)]
struct Node {
    id: NodeId,
    metadata: Vec<NodeId>, // 1 or more
    children: Vec<NodeId>, // 0 or more
}

impl Node {
    // runs through the iterator, parsing the nodes into Node structs. Returns the id of the root
    // node, a HashMap of the Node structs, and what's left of the iterator.

    // Is there a better option besides having to use an external iterator, transferring its
    // ownership, and having to return it?

    fn parse(
        mut iter: SplitAsciiWhitespace,
        mut id: NodeId,
    ) -> Result<(
        NodeId,
        HashMap<NodeId, Self>,
        SplitAsciiWhitespace,
    )> {
        if let (Some(children_str), Some(metadata_str)) = (iter.next(), iter.next()) {
            let (num_children, num_metadata) =
                (children_str.parse::<u32>()?, metadata_str.parse::<usize>()?);
            let curr_node_id = id;
            id += 1;
            let mut nodes = HashMap::<NodeId, Node>::new();
            let mut children = vec![];
            // TODO: do this without a loop?
            for _ in 0..num_children {
                let (child_node_id, new_nodes, next_iter) = Node::parse(iter, id)?;
                iter = next_iter; // re-assign the input for the next iteration
                id += new_nodes.len() as u32;
                children.push(child_node_id);

                // Ideally, we'd use HashMap.extend, but we want to make sure we aren't overwriting anything here.
                new_nodes.into_iter().for_each(|(node_id, node)| {
                    if let Some(old_node) = nodes.insert(node_id, node) {
                        // TODO: error instead of panic:
                        panic!("collision when inserting node: {:?}", old_node);
                    }
                })
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
            if let Some(node) = nodes.insert(node.id, node) {
                panic!("overwriting node id: {}", node.id);
            }
            Ok((curr_node_id, nodes, iter))
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

type Sum = u64;

impl Tree {
    fn parse(input: &str) -> Result<Self> {
        let (root, nodes, mut iter) = Node::parse(input.split_ascii_whitespace(), 0)?;
        if let Some(_) = iter.next() {
            panic!("iter should be empty now.");
        }
        Ok(Tree { nodes, root })
    }

    // Part 1
    fn sum_metadata(&self) -> u32 {
        self.nodes
            .values()
            .flat_map(|node| node.metadata.clone())
            .sum::<u32>()
    }

    // Part 2
    fn get_root_value(&self) -> Sum {
        let cache = HashMap::<NodeId, Sum>::new();
        self._get_value(self.root, cache).0
    }

    // Return the value for a given NodeId
    // While also maintaining a cache for the lookups...
    fn _get_value(
        &self,
        id: NodeId,
        mut cache: HashMap<NodeId, Sum>,
    ) -> (Sum, HashMap<NodeId, Sum>) {
        if let Some(&value) = cache.get(&id) {
            return (value, cache);
        }

        let node = self
            .nodes
            .get(&id)
            .expect(&format!("invalid node id: {}", id));

        let num_children = node.children.len() as u32;
        let value = if num_children == 0 {
            // get sum of node's metadata:
            node.metadata.iter().map(|&id| u64::from(id)).sum::<Sum>()
        } else {
            // get value of the node's children:
            // let mut temp_cache = std::mem::replace(&mut cache, HashMap::new());
            let mut sum = 0;
            // TODO: how to do this without a for loop? (see iterator below)
            for &i in node.metadata.iter() {
                if 1 <= i && i <= num_children {
                    // recursive case
                    let node_id: NodeId = node.children[(i - 1) as usize];
                    let (node_value, new_cache_2) = self._get_value(node_id, cache);
                    cache = new_cache_2;
                    sum += node_value;
                    // } else {
                    //     // if i is out of range of the nodes children, then map it to 0
                    //     0
                }
            }
            // let v = node.metadata
            //     .iter()
            //     .map(move |&i| {
            //         if 1 <= i && i <= num_children {
            //             // recursive case
            //             let node_id: NodeId = node.children[(i - 1) as usize];
            //             let (node_value, new_cache_2) = self._get_value(node_id, temp_cache);
            //             temp_cache = new_cache_2;
            //             node_value
            //         } else {
            //             // if i is out of range of the nodes children, then map it to 0
            //             0
            //         }
            //     })
            //     .sum();
            // std::mem::replace(&mut cache, temp_cache);
            // cache = temp_cache;
            sum
        };
        cache.insert(id, value);
        (value, cache)
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sorted_nodes = self.nodes.values().collect::<Vec<&Node>>();
        sorted_nodes.sort_by(|&node_1, &node_2| node_1.id.cmp(&node_2.id));

        write!(
            f,
            "{}",
            sorted_nodes
                .iter()
                .map(|n| format!("{:?}", n))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[test]
fn test_metadata_sum() -> Result<()> {
    let input = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";
    let tree = Tree::parse(&input)?;
    assert_eq!(tree.sum_metadata(), 138);
    assert_eq!(
        tree.nodes.keys().collect::<HashSet<&NodeId>>(),
        HashSet::<&NodeId>::from_iter(vec![0, 1, 2, 3].iter())
    );
    assert_eq!(tree.nodes.get(&0).unwrap().children, vec![1, 2]);
    assert_eq!(tree.nodes.get(&1).unwrap().children, vec![]);
    assert_eq!(tree.nodes.get(&2).unwrap().children, vec![3]);
    assert_eq!(tree.nodes.get(&3).unwrap().children, vec![]);
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

#[test]
fn test_hashmap_extends() -> Result<()> {
    let mut map1 = HashMap::<NodeId, &str>::new();
    let mut map2 = HashMap::<NodeId, &str>::new();
    map1.insert(1, "1");
    map2.insert(2, "2");
    map2.insert(1, "5");

    // Note: ideally, we should be able to know when something is being overwitten here...

    map1.extend(map2.into_iter());

    assert_eq!(map1.get(&1), Some(&"5"));
    assert_eq!(map1.get(&2), Some(&"2"));
    println!("test_root_node_value passed.");
    Ok(())
}

#[test]
fn test_root_node_value() -> Result<()> {
    let input = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";
    let tree = Tree::parse(&input)?;
    assert_eq!(tree.get_root_value(), 66);
    println!("test_root_node_value passed.");
    Ok(())
}
