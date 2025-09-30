// ####################### Node structure for reference: ##################################################
// pub struct Node {
//     pub s_node_id: u16,
//     pub s_qty: u16,
//     pub d_node_id: u16,
//     pub d_qty: u16,
//     pub node_cost: f32,
//     pub node_qty: u16,
// }

use crate::node;
use ndarray::Array;
use ndarray::Dim;

pub fn greedy (task_vec: &mut Vec<node::Node,>, task_array2d: &mut Array<node::Node, Dim<[usize; 2]>>) {
    // Sorting nodes vec by cost with custom comparator function
    task_vec.sort_by_key(|node| node.node_cost as i32);
    
    // let task_size: usize = task_vec.len();
    let s_size: usize = task_array2d.nrows();
    let d_size: usize = task_array2d.ncols();

    // perform greedy algorithm
    for node in task_vec.iter() {
        let s_idx = node.s_node_id as usize;
        let d_idx = node.d_node_id as usize;

        let s_qty = task_array2d[(s_idx, d_idx)].s_qty;
        let d_qty = task_array2d[(s_idx, d_idx)].d_qty;

        if s_qty >= d_qty && d_qty != 0 {
            for j in 0..d_size{
                task_array2d[(s_idx, j)].s_qty = s_qty - d_qty;
            }
            for i in 0..s_size{
                task_array2d[(i, d_idx)].d_qty = 0;
            }
            task_array2d[(s_idx, d_idx)].node_qty = d_qty;
        }
        else if d_qty > s_qty && s_qty != 0 {
            for j in 0..d_size{
                task_array2d[(s_idx, j)].s_qty = 0;
            }
            for i in 0..s_size{
                task_array2d[(i, d_idx)].d_qty = d_qty - s_qty;
            }
            task_array2d[(s_idx, d_idx)].node_qty = s_qty;
        }
    }
}