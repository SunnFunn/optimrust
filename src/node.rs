#[allow(dead_code)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Node {
    pub s_node_id: usize,
    pub s_qty: i32,
    pub d_node_id: usize,
    pub d_qty: i32,
    pub node_cost: i32,
    pub node_qty: i32,
}

impl Node {
    pub fn new() -> Node{
        Node {
            s_node_id: 0,
            s_qty: 0,
            d_node_id: 0,
            d_qty: 0,
            node_cost: 0,
            node_qty: 0,
        }
    }

    pub fn new_with_data(s_id: &str, d_id: &str, s_qt: i32, d_qt: i32, cost: &str) -> Node{
        Node {
            s_node_id: s_id.parse().unwrap(),
            s_qty: s_qt,
            d_node_id: d_id.parse().unwrap(),
            d_qty: d_qt,
            node_cost: cost.parse().unwrap(),
            node_qty: 0,
        }
    }
}

