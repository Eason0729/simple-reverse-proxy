#[derive(Debug)]
pub struct Tree<T> {
    len: usize,
    children: Vec<Vec<usize>>,
    value: Vec<T>,
}

#[derive(Clone, Copy, Debug)]
pub struct Node {
    index: usize,
}

impl<T> Tree<T>
where
    T: Default,
{
    pub fn new() -> Self {
        Tree {
            len: 1,
            children: vec![vec![]],
            value: vec![T::default()],
        }
    }
    pub fn add_node(&mut self, value: T) -> Node {
        let index = self.len;
        self.len += 1;
        self.children.push(vec![]);
        self.value.push(value);
        Node { index }
    }
    pub fn root(&mut self) -> Node {
        Node { index: 0 }
    }
}

impl Node {
    pub fn add_child<T>(&mut self, tree: &mut Tree<T>, child: Node) {
        let parent_index = self.index;
        let child_index = child.index;
        tree.children[parent_index].push(child_index);
    }
    pub fn set_value<'a, T>(&self, tree: &'a mut Tree<T>, value: T) {
        let index = self.index;
        tree.value[index] = value;
    }
    pub fn value<'a, T>(&self, tree: &'a mut Tree<T>) -> &'a T {
        let index = self.index;
        &tree.value[index]
    }
    pub fn children<'a, T>(&self, tree: &'a mut Tree<T>) -> Vec<Node> {
        let index = self.index;
        tree.children[index]
            .clone()
            .into_iter()
            .map(|index| Node { index })
            .collect()
    }
}
