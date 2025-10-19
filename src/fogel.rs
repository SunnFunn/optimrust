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
use ndarray::{Array, Slice, s};
use ndarray::Dim;
use rayon::prelude::*;
// use ndarray::parallel::prelude::*;

// ####################### Fogel algo: ##################################################

pub fn fogel (task_array2d: &mut Array<node::Node, Dim<[usize; 2]>>) {
    
    // define problem sizes
    let s_size: usize = task_array2d.nrows();
    let d_size: usize = task_array2d.ncols();

    // define vector of excluded row && col idxs
    let mut row_excld = Vec::new();
    let mut col_excld = Vec::new();
    let mut nodes_excld = Vec::new();

    // loop until check of supply || demand left unassigned get true (no left demand or supply)
    // let mut check: bool = true;
    let mut count: i16 = 0;
    loop {
        count += 1;
        // define row && columns minimal costs diffs
        //---------------------------------------------------------------------------------------
        let mut row_diffs: Vec<Vec<i32>> = Vec::new();
        let mut col_diffs: Vec<Vec<i32>> = Vec::new();

        // define row diffs for not excluded rows
        for row in task_array2d.rows() {
            let mut row_vec = row.to_vec();
            if row_excld.contains(&row_vec[0].s_node_id) { continue; }
            // if row_vec[0].s_qty == 0 { continue; }

            // row_vec.retain(|&x| x.node_qty == 0);
            row_vec.retain(|&x| !nodes_excld.contains(&vec![x.s_node_id, x.d_node_id]));
            if row_vec.len() == 0 { continue; }
            row_vec.par_sort_by(|a, b| a.node_cost.partial_cmp(&b.node_cost).unwrap());
            
            let mut diff: i32 = 0;
            if row_vec.len() > 1 {
                diff = row_vec[1].node_cost - row_vec[0].node_cost;
            }
            else {
                diff = row_vec[0].node_cost;
            }
            
            let data = vec![diff, row_vec[0].s_node_id as i32];
            row_diffs.push(data);
        }

        // define col diffs for not excluded columns
        for col in task_array2d.columns() {
            let mut col_vec = col.to_vec();
            if col_excld.contains(&col_vec[0].d_node_id) { continue; }
            // if col_vec[0].d_qty == 0 { continue; }

            // col_vec.retain(|&x| x.node_qty == 0);
            col_vec.retain(|&x| !nodes_excld.contains(&vec![x.s_node_id, x.d_node_id]));
            if col_vec.len() == 0 { continue; }
            col_vec.par_sort_by(|a, b| a.node_cost.partial_cmp(&b.node_cost).unwrap());
            
            let mut diff: i32 = 0;
            if col_vec.len() > 1 {
                diff = col_vec[1].node_cost - col_vec[0].node_cost;
            }
            else {
                diff = col_vec[0].node_cost;
            }
            let data = vec![diff, col_vec[0].d_node_id as i32];
            col_diffs.push(data);
        }
        // println!("{:?}  {:?}", row_diffs, col_diffs);

        // calculate max values of row && col diffs
        let mut max_row_diff = vec![-1, -1];
        let mut max_col_diff = vec![-1, -1];

        if row_diffs.len() >= 1 {
            row_diffs.par_sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
            max_row_diff[0] = row_diffs[row_diffs.len() - 1][0];
            max_row_diff[1] = row_diffs[row_diffs.len() - 1][1];
        }
        if col_diffs.len() >= 1 {
            col_diffs.par_sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
            max_col_diff[0] = col_diffs[col_diffs.len() - 1][0];
            max_col_diff[1] = col_diffs[col_diffs.len() - 1][1];
        }
        
        // println!("{:?}  {:?}", max_row_diff, max_col_diff);

        // define node to assign && assign qtys to this node
        //---------------------------------------------------------------------------------
        
        // find node idxs
        let mut s_idx: usize = 0;
        let mut d_idx: usize = 0;

        if max_row_diff[0] == -1 && max_col_diff[0] == -1 { break; }

        if max_row_diff[0] != -1  && max_row_diff[0] >= max_col_diff[0] {
            s_idx = max_row_diff[1] as usize;
            // row_excld.push(s_idx);
            
            let mut row_slice = task_array2d.slice(s![s_idx, ..]).clone().to_vec();
            // row_slice.retain(|&x| x.node_qty == 0);
            row_slice.retain(|&x| !nodes_excld.contains(&vec![x.s_node_id, x.d_node_id]));
            row_slice.par_sort_by(|a, b| a.node_cost.partial_cmp(&b.node_cost).unwrap());

            d_idx = row_slice[0].d_node_id;
        }
        else if max_col_diff[0] != -1  && max_col_diff[0] > max_row_diff[0] {
            d_idx = max_col_diff[1] as usize;
            // col_excld.push(d_idx);

            let mut col_slice = task_array2d.slice(s![.., d_idx]).clone().to_vec();
            // col_slice.retain(|&x| x.node_qty == 0);
            col_slice.retain(|&x| !nodes_excld.contains(&vec![x.s_node_id, x.d_node_id]));
            col_slice.par_sort_by(|a, b| a.node_cost.partial_cmp(&b.node_cost).unwrap());

            s_idx = col_slice[0].s_node_id;
        }

        nodes_excld.push(vec![s_idx, d_idx]);

        // println!("{:?}  {:?}", s_idx, d_idx);

        // perform node filling algorithm
        let s_qty = task_array2d[(s_idx, d_idx)].s_qty;
        let d_qty = task_array2d[(s_idx, d_idx)].d_qty;

        if s_qty >= d_qty && d_qty != 0 {
            for j in 0..d_size{
                task_array2d[(s_idx, j)].s_qty = s_qty - d_qty;
            }
            for i in 0..s_size{
                task_array2d[(i, d_idx)].d_qty = 0;
            }
            col_excld.push(d_idx);
            task_array2d[(s_idx, d_idx)].node_qty = d_qty;
        }
        else if d_qty > s_qty && s_qty != 0 {
            for j in 0..d_size{
                task_array2d[(s_idx, j)].s_qty = 0;
            }
            row_excld.push(s_idx);
            for i in 0..s_size{
                task_array2d[(i, d_idx)].d_qty = d_qty - s_qty;
            }
            task_array2d[(s_idx, d_idx)].node_qty = s_qty;
        }

        // println!("{:?}  {:?} {:?}", row_excld, col_excld, nodes_excld);
        // println!("{:?}", task_array2d);
        // check if loop has to be stopped
        let mut s_total = 0;
        let mut d_total = 0;
        for row in task_array2d.rows() {
            s_total += row[0].s_qty;
        }
        for col in task_array2d.columns() {
            d_total += col[0].d_qty;
        }
        if s_total == 0 && d_total == 0 { break; }

        // if count == 20 { break; }
    }
}