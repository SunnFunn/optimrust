#[allow(dead_code)]
use redis::{Client,Commands};
use std::time::Instant;
use ndarray::Array;
// use ndarray::ShapeBuilder;

mod node;
mod greedy;
mod simplex;
mod getdata;


fn main () {
    let now1 = Instant::now();

    // rset connection to redis DB
    //-------------------------------------------------------------------------------------
    let client = Client::open("redis://:alext@127.0.0.1:6379/").unwrap();
    let mut connection = client.get_connection().unwrap();

    // get task vector & task sizes
    //-------------------------------------------------------------------------------------
    let optim_data = getdata::get().unwrap();
    let mut optim_task_vec = optim_data.0;
    let s_task_size = optim_data.1;
    let d_task_size = optim_data.2;

    // Initialize task matrix as 2D array from task vector
    //-------------------------------------------------------------------------------------
    let mut optim_task_array2d = Array::from_vec(optim_task_vec.clone())
                                .clone()
                                .into_shape_with_order((s_task_size, d_task_size))
                                .unwrap();

    // println!("{:?}", optim_task_array2d);

    // Perform greedy optimization
    //-------------------------------------------------------------------------------------
    greedy::greedy(&mut optim_task_vec, &mut optim_task_array2d);
    
    // Printout results
    //-------------------------------------------------------------------------------------
    let mut s_total = 0;
    let mut d_total = 0;
    let mut total_asiignment_qty = 0;
    let mut total_cost = 0;

    for row in optim_task_array2d.rows() {
        for col in row.iter() {
            s_total += col.s_qty;
            d_total += col.d_qty;
            total_asiignment_qty += col.node_qty;
            total_cost += col.node_qty*(col.node_cost as i32);
        }
    }
    
    println!("Total assignment qty: {}", total_asiignment_qty);
    println!("Total problem cost: {}", total_cost);
    println!("Total left supply: {}", s_total);
    println!("Total left demand: {}", d_total);

    let elapsed = now1.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    // // Initial feasible solution for CBC solver
    // //-------------------------------------------------------------------------------------
    // let mut costs_to_redis: String = "".to_string();
    // for row_iter in optim_task_arr.rows_iter() {
    //     for col in row_iter {
    //         costs_to_redis += &(col.node_qty.to_string());
    //         costs_to_redis += "_";
    //     }
    // } 
    // let _: () = connection.set("initial_solve", &costs_to_redis).unwrap();

    // // Initial basis forming
    // //-------------------------------------------------------------------------------------
    // let mut initial_basis_vec: Vec<usize> = Vec::<usize>::new();

    // for row_iter in optim_task_arr.rows_iter() {
    //     for node in row_iter {
    //         let node_id = node.s_node_id*optim_task_arr.num_columns() + node.d_node_id;
    //         if node.node_qty != 0 {
    //             initial_basis_vec.push(node_id);
    //         }
    //         print!("node_id: {} qty: {}  ", node_id, node.node_qty);
    //     }
    // }
    // println!("{:?}", initial_basis_vec);

    // Perform simplex optimization
    //-------------------------------------------------------------------------------------
    // simplex::simplex_optimize();
}