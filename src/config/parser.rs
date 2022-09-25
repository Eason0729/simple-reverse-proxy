use super::level::*;
use super::tree::*;
use std::cmp;
use std::io;

const MAX_DEPTH: usize = 32;

fn gcd(a: usize, b: usize) -> usize {
    let mut d = 1;
    let mut i = 2;
    while i <= cmp::min(a, b) {
        if 0 == a % i && 0 == b % i {
            d = i;
        };
        i += 1;
    }
    d
}

struct Line {
    offset: usize,
    value: String,
}

pub struct Parser {
    lines: Vec<Line>,
}

impl Parser {
    pub fn new<S>(stream: S) -> Self
    where
        S: io::Read + io::BufRead,
    {
        let lines: Vec<_> = stream.lines().map(|s| s.unwrap()).collect();
        // collect lines into tuple(number of padding space, rest of lines)
        let lines: Vec<(usize, String)> = lines
            .iter()
            .map(|s| {
                let org_len = s.len();
                let content = s.trim_start();
                let padding = org_len - content.len();
                (padding, content.to_string())
            })
            .collect();
        // compute the offset of yml stream
        let mut offset = 1 * 2 * 3 * 4 * 5 * 6;
        lines.iter().for_each(|(p, _)| {
            if *p != 0 {
                offset = gcd(offset, *p);
            }
        });
        // map lines into Line
        let lines: Vec<Line> = lines
            .into_iter()
            .map(|mut content| {
                content.0 /= offset;
                #[cfg(debug_assertions)]
                assert!(MAX_DEPTH > content.0);
                Line {
                    offset: content.0,
                    value: content.1,
                }
            })
            .collect();
        Parser { lines }
    }

    fn tree(self) -> Tree<String> {
        let mut tree = Tree::<String>::new();
        let mut parents = vec![tree.root(); MAX_DEPTH];
        for line in self.lines.iter() {
            let offset = line.offset;
            let node = tree.add_node(line.value.clone());
            parents[offset].add_child(&mut tree, node);
            parents[offset + 1] = node;
        }
        tree
    }

    pub fn parse(self) -> Level {
        let tree = &mut self.tree();
        tree.root().set_value(tree, "root:".to_string());

        fn recursive_parsing(node: Node, tree: &mut Tree<String>) -> Level {
            let value = node.value(tree).clone();
            if value.trim_end().ends_with(":") {
                let value=value.strip_suffix(":").unwrap();
                let children: Vec<Level> = node
                    .children(tree)
                    .into_iter()
                    .map(|node| recursive_parsing(node, tree))
                    .collect();
                return Level::Level(value.to_string(), children);
            }
            if value.starts_with("-") {
                return Level::List(value.strip_prefix("-").unwrap().into());
            }
            if value.ends_with("]") {
                let (value, field) = value.split_once(":").unwrap();
                let lists: Vec<Value> = field
                    .split(",")
                    .map(|source| source.trim())
                    .map(|source| source.into())
                    .collect();
                let lists = lists.into_iter().map(|x| Level::List(x)).collect();
                return Level::Level(value.to_string(), lists);
            }

            let (value, field) = value.split_once(":").unwrap();
            Level::Level(value.to_string(), vec![Level::Unspecified(field.into())])
        }

        recursive_parsing(tree.root(), tree)
    }
}

// Level,        routing:
// Array,        routing: ["a.example.com"]
// List,         - a.example.com
// Bool,         rewrite: true
// Number,       weight: 1
// Unspecified,  value: "ABC"

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn lists() {
        let file = fs::File::open("test/defaultyml").unwrap();
        let reader = io::BufReader::new(file);
        let parser = Parser::new(reader);
        let root = parser.parse();

        let lists: Vec<String> = root
            .list(vec!["hosts", "a.example.com", "routing"])
            .unwrap()
            .into_iter()
            .map(|x| x.try_into().unwrap())
            .collect();
        assert_eq!(vec!["127.0.0.1:8000", "b.example.com:8001",], lists)
    }

    #[test]
    fn value_f64() {
        let file = fs::File::open("test/simpleyml").unwrap();
        let reader = io::BufReader::new(file);
        let parser = Parser::new(reader);
        let root = parser.parse();

        let val: f64 = root.value(vec!["a"]).unwrap().try_into().unwrap();
        assert_eq!(val, 1.3);
    }
}
