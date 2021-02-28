use std::{cmp::Ordering, fmt::Debug};

pub trait KdValue: Default + Clone + Debug + PartialEq {
    type Position: PartialOrd + Debug;
    fn min_x(&self) -> Self::Position;
    fn min_y(&self) -> Self::Position;
    fn max_x(&self) -> Self::Position;
    fn max_y(&self) -> Self::Position;
}
#[derive(Debug)]
pub enum KdTree<Value: KdValue, const ISLAND_SIZE: usize> {
    Leaf(Vec<Value>),
    Node(Box<KdNode<Value, ISLAND_SIZE>>),
}

impl<Value: KdValue, const ISLAND_SIZE: usize> Default for KdTree<Value, ISLAND_SIZE> {
    fn default() -> Self {
        Self::Leaf(Vec::with_capacity(ISLAND_SIZE))
    }
}

impl<Value: KdValue, const ISLAND_SIZE: usize> KdTree<Value, ISLAND_SIZE> {
    pub fn insert(&mut self, value: Value) {
        self.insert_internal(value, false)
    }

    pub fn remove_one(&mut self, value: Value) -> bool {
        match self {
            KdTree::Leaf(leaf) => {
                let index = leaf
                    .iter()
                    .enumerate()
                    .find(|(_, val)| val == &&value)
                    .map(|t| t.0);
                if let Some(index) = index {
                    leaf.swap_remove(index);
                    true
                } else {
                    false
                }
            }
            KdTree::Node(node) => node.remove_one(value),
        }
    }

    pub fn remove_all(&mut self, value: Value) {
        match self {
            KdTree::Leaf(leaf) => {
                let indexes: Vec<usize> = leaf
                    .iter()
                    .enumerate()
                    .filter(|(_, val)| **val == value)
                    .map(|t| t.0)
                    .collect();
                for index in indexes {
                    leaf.swap_remove(index);
                }
            }
            KdTree::Node(node) => node.remove_all(value),
        }
    }

    fn insert_internal(&mut self, value: Value, vertical: bool) {
        let change = match self {
            KdTree::Leaf(leaf) => {
                assert!(leaf.len() < ISLAND_SIZE);
                leaf.push(value);
                if leaf.len() < ISLAND_SIZE {
                    None
                } else {
                    leaf.sort_unstable_by(if vertical {
                        |a: &Value, b: &Value| {
                            a.min_y().partial_cmp(&b.min_y()).unwrap_or(Ordering::Equal)
                        }
                    } else {
                        |a: &Value, b: &Value| {
                            a.min_x().partial_cmp(&b.min_x()).unwrap_or(Ordering::Equal)
                        }
                    });
                    let median = if vertical {
                        leaf[ISLAND_SIZE / 2].clone().min_y()
                    } else {
                        leaf[ISLAND_SIZE / 2].clone().min_x()
                    };
                    let right = KdTree::Leaf(leaf.split_off(ISLAND_SIZE / 2));
                    let left = std::mem::take(leaf);
                    let init = if vertical {
                        left[0].max_y()
                    } else {
                        left[0].max_x()
                    };
                    let left_max = left.iter().fold(init, |prev, value| {
                        let v_max = if vertical {
                            value.max_y()
                        } else {
                            value.max_x()
                        };
                        if v_max > prev {
                            v_max
                        } else {
                            prev
                        }
                    });
                    let left = KdTree::Leaf(left);
                    Some(KdTree::Node(Box::new(KdNode {
                        left,
                        right,
                        median,
                        vertical,
                        left_max,
                    })))
                }
            }
            KdTree::Node(node) => {
                node.insert(value);
                None
            }
        };
        if let Some(new_tree) = change {
            *self = new_tree;
        }
    }
    //false positive it seems
    #[allow(clippy::needless_lifetimes)]
    pub fn query_point<'a>(
        &'a self,
        x: Value::Position,
        y: Value::Position,
    ) -> PointQuery<'a, Value, ISLAND_SIZE> {
        PointQuery::new(self, x, y)
    }
    //false positive it seems
    #[allow(clippy::needless_lifetimes)]
    pub fn query_rect<'a>(
        &'a self,
        min_x: Value::Position,
        max_x: Value::Position,
        min_y: Value::Position,
        max_y: Value::Position,
    ) -> RectQuery<'a, Value, ISLAND_SIZE> {
        RectQuery::new(self, min_x, max_x, min_y, max_y)
    }
}
pub struct RectQuery<'a, Value: KdValue, const ISLAND_SIZE: usize> {
    max_x: Value::Position,
    min_x: Value::Position,
    max_y: Value::Position,
    min_y: Value::Position,
    queue: Vec<&'a KdTree<Value, ISLAND_SIZE>>,
    items_to_yield: Vec<&'a Value>,
}
impl<'a, Value: KdValue, const ISLAND_SIZE: usize> RectQuery<'a, Value, ISLAND_SIZE> {
    fn new(
        tree: &'a KdTree<Value, ISLAND_SIZE>,
        min_x: Value::Position,
        max_x: Value::Position,
        min_y: Value::Position,
        max_y: Value::Position,
    ) -> Self {
        Self {
            queue: vec![tree],
            items_to_yield: Vec::new(),
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }
}
impl<'a, Value: KdValue, const ISLAND_SIZE: usize> Iterator for RectQuery<'a, Value, ISLAND_SIZE> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items_to_yield.pop();
        if item.is_some() {
            return item;
        }
        loop {
            if self.queue.is_empty() {
                return None;
            }
            let tree = self.queue.pop().unwrap();
            match tree {
                KdTree::Leaf(leaves) => {
                    for leaf in leaves {
                        if !(leaf.min_x() > self.max_x
                            || self.min_x > leaf.max_x()
                            || leaf.min_y() > self.max_y
                            || self.min_y > leaf.max_y())
                        {
                            self.items_to_yield.push(leaf)
                        }
                    }
                    let item = self.items_to_yield.pop();
                    if item.is_some() {
                        return item;
                    }
                }
                KdTree::Node(node) => {
                    let (min, max) = if node.vertical {
                        (&self.min_y, &self.max_y)
                    } else {
                        (&self.min_x, &self.max_x)
                    };
                    if *min <= node.left_max {
                        self.queue.push(&node.left)
                    }
                    if *max >= node.median {
                        self.queue.push(&node.right)
                    }
                }
            }
        }
    }
}
pub struct PointQuery<'a, Value: KdValue, const ISLAND_SIZE: usize> {
    x: Value::Position,
    y: Value::Position,
    queue: Vec<&'a KdTree<Value, ISLAND_SIZE>>,
    items_to_yield: Vec<&'a Value>,
}
impl<'a, Value: KdValue, const ISLAND_SIZE: usize> PointQuery<'a, Value, ISLAND_SIZE> {
    fn new(tree: &'a KdTree<Value, ISLAND_SIZE>, x: Value::Position, y: Value::Position) -> Self {
        Self {
            queue: vec![tree],
            items_to_yield: Vec::new(),
            x,
            y,
        }
    }
}
impl<'a, Value: KdValue, const ISLAND_SIZE: usize> Iterator for PointQuery<'a, Value, ISLAND_SIZE> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items_to_yield.pop();
        if item.is_some() {
            return item;
        }
        loop {
            if self.queue.is_empty() {
                return None;
            }
            let tree = self.queue.pop().unwrap();
            match tree {
                KdTree::Leaf(leaves) => {
                    for leaf in leaves {
                        if leaf.min_x() <= self.x
                            && leaf.max_x() >= self.x
                            && leaf.min_y() <= self.y
                            && leaf.max_y() >= self.y
                        {
                            self.items_to_yield.push(leaf)
                        }
                    }
                    let item = self.items_to_yield.pop();
                    if item.is_some() {
                        return item;
                    }
                }
                KdTree::Node(node) => {
                    let dim = if node.vertical { &self.y } else { &self.x };
                    if *dim <= node.left_max {
                        self.queue.push(&node.left)
                    }
                    if *dim >= node.median {
                        self.queue.push(&node.right)
                    }
                }
            }
        }
    }
}
#[derive(Debug)]
pub struct KdNode<Value: KdValue, const ISLAND_SIZE: usize> {
    vertical: bool,
    median: Value::Position,
    left_max: Value::Position,
    left: KdTree<Value, ISLAND_SIZE>,
    right: KdTree<Value, ISLAND_SIZE>,
}

impl<Value: KdValue, const ISLAND_SIZE: usize> KdNode<Value, ISLAND_SIZE> {
    fn choose_tree(&mut self, value: &Value) -> &mut KdTree<Value, ISLAND_SIZE> {
        let cmp_position = if self.vertical {
            value.min_y()
        } else {
            value.min_x()
        };
        if cmp_position < self.median {
            let max = if self.vertical {
                value.max_y()
            } else {
                value.max_x()
            };
            if max > self.left_max {
                self.left_max = max
            }
            &mut self.left
        } else {
            &mut self.right
        }
    }
    fn insert(&mut self, value: Value) {
        let vertical = self.vertical;
        self.choose_tree(&value).insert_internal(value, !vertical);
    }
    fn remove_one(&mut self, value: Value) -> bool {
        self.choose_tree(&value).remove_one(value)
    }
    fn remove_all(&mut self, value: Value) {
        self.choose_tree(&value).remove_all(value);
    }
}

#[cfg(test)]
mod tests {
    use core::f32;

    use crate::{KdTree, KdValue};
    #[derive(Debug, Default, Clone, PartialEq)]
    struct TestValue {
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    }
    impl TestValue {
        fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
            Self {
                min_x,
                max_x,
                min_y,
                max_y,
            }
        }
    }
    impl KdValue for TestValue {
        type Position = f32;
        fn min_x(&self) -> Self::Position {
            self.min_x
        }

        fn min_y(&self) -> Self::Position {
            self.min_y
        }

        fn max_x(&self) -> Self::Position {
            self.max_x
        }

        fn max_y(&self) -> Self::Position {
            self.max_y
        }
    }
    #[test]
    fn rect() {
        let mut tree = KdTree::<TestValue, 3>::default();
        tree.insert(TestValue::new(3., 5., 4., 6.));
        tree.insert(TestValue::new(4., 6., 7., 9.));
        tree.insert(TestValue::new(6., 10., 3., 7.));
        tree.insert(TestValue::new(7., 8., 4., 5.));
        tree.insert(TestValue::new(6., 8., 1., 3.));
        tree.insert(TestValue::new(3., 5., 4., 6.));
        tree.insert(TestValue::new(4., 6., 7., 9.));
        tree.insert(TestValue::new(6., 10., 3., 7.));
        tree.insert(TestValue::new(7., 8., 4., 5.));
        tree.insert(TestValue::new(6., 8., 1., 3.));
        tree.insert(TestValue::new(3., 5., 4., 6.));
        tree.insert(TestValue::new(4., 6., 7., 9.));
        tree.insert(TestValue::new(6., 10., 3., 7.));
        tree.insert(TestValue::new(7., 8., 4., 5.));
        tree.insert(TestValue::new(6., 8., 1., 3.));
        assert_eq!(tree.query_rect(5.5, 7.5, 3.5, 7.5).count(), 9);
    }
    #[test]
    fn point() {
        let mut tree = KdTree::<TestValue, 4>::default();
        tree.insert(TestValue::new(3., 5., 4., 6.));
        tree.insert(TestValue::new(4., 6., 7., 9.));
        tree.insert(TestValue::new(6., 10., 3., 7.));
        tree.insert(TestValue::new(7., 8., 4., 5.));
        tree.insert(TestValue::new(6., 8., 1., 3.));
        tree.insert(TestValue::new(3., 5., 4., 6.));
        tree.insert(TestValue::new(4., 6., 7., 9.));
        tree.insert(TestValue::new(6., 10., 3., 7.));
        tree.insert(TestValue::new(7., 8., 4., 5.));
        tree.insert(TestValue::new(6., 8., 1., 3.));
        tree.insert(TestValue::new(3., 5., 4., 6.));
        tree.insert(TestValue::new(4., 6., 7., 9.));
        tree.insert(TestValue::new(6., 10., 3., 7.));
        tree.insert(TestValue::new(7., 8., 4., 5.));
        tree.insert(TestValue::new(6., 8., 1., 3.));
        assert_eq!(tree.query_point(7.5, 4.5).count(), 6);
    }
}
