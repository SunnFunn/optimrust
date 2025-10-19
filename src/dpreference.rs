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
use rayon::prelude::*;
// use ndarray::parallel::prelude::*;

// ####################### Double Preference algo: ##################################################
pub fn dpreference (task_array2d: &mut Array<node::Node, Dim<[usize; 2]>>) {
    let task_array2d_copy = task_array2d.clone();
    // fill preference idxs in rows
    for row in task_array2d_copy.rows() {
            let mut row_vec = row.to_vec();
            row_vec.par_sort_by(|a, b| a.node_cost.partial_cmp(&b.node_cost).unwrap());
            for (i, ele) in row_vec.iter().enumerate() {
                if i == 0 {
                    task_array2d[(row_vec[i].s_node_id, row_vec[i].d_node_id)].dpref -= 1;
                }
                else if i > 0 && row_vec[i].node_cost == row_vec[0].node_cost {
                    task_array2d[(row_vec[i].s_node_id, row_vec[i].d_node_id)].dpref -= 1;
                }
                else { continue; }
            }
        }
    
    // fill preference idxs in columns
    for col in task_array2d_copy.columns() {
            let mut col_vec = col.to_vec();
            col_vec.par_sort_by(|a, b| a.node_cost.partial_cmp(&b.node_cost).unwrap());
            for (i, ele) in col_vec.iter().enumerate() {
                if i == 0 {
                    task_array2d[(col_vec[i].s_node_id, col_vec[i].d_node_id)].dpref -= 1;
                }
                else if i > 0 && col_vec[i].node_cost == col_vec[0].node_cost {
                    task_array2d[(col_vec[i].s_node_id, col_vec[i].d_node_id)].dpref -= 1;
                }
                else { continue; }
            }
        }

    let mut task_vec = task_array2d.clone().into_raw_vec_and_offset().0;
    task_vec.par_sort_unstable_by_key(|item| (item.dpref, item.node_cost));

    // println!("{:?}", &task_vec);
    
    // let task_size: usize = task_vec.len();
    let s_size: usize = task_array2d.nrows();
    let d_size: usize = task_array2d.ncols();

    // perform greedy algorithm
    for node in task_vec.into_iter() {
        let s_idx = node.s_node_id as usize;
        let d_idx = node.d_node_id as usize;

        let s_qty = task_array2d[(s_idx, d_idx)].s_qty;
        let d_qty = task_array2d[(s_idx, d_idx)].d_qty;

        if s_qty >= d_qty && d_qty != 0 {
            // task_array2d.row_mut(s_idx).par_map_inplace(|node| node.s_qty = s_qty - d_qty);
            // task_array2d.column_mut(d_idx).par_map_inplace(|node| node.d_qty = 0);

            for j in 0..d_size{
                task_array2d[(s_idx, j)].s_qty = s_qty - d_qty;
            }
            for i in 0..s_size{
                task_array2d[(i, d_idx)].d_qty = 0;
            }
            task_array2d[(s_idx, d_idx)].node_qty = d_qty;
        }
        else if d_qty > s_qty && s_qty != 0 {
            // task_array2d.row_mut(s_idx).par_map_inplace(|node| node.s_qty = 0);
            // task_array2d.column_mut(d_idx).par_map_inplace(|node| node.d_qty = d_qty - s_qty);

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