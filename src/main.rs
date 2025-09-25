#[allow(dead_code)]
use redis::{Client,Commands};
use std::time::Instant;
use array2d::{Array2D, Error};

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
    let mut optim_task_arr = Array2D::from_row_major(&optim_task_vec, s_task_size, d_task_size).unwrap();

    // Perform greedy optimization
    //-------------------------------------------------------------------------------------
    greedy::greedy(&mut optim_task_arr, &mut optim_task_vec);
    
    // Printout results
    //-------------------------------------------------------------------------------------
    let mut s_total = 0;
    let mut d_total = 0;
    let mut total_asiignment_qty = 0;
    let mut total_cost = 0;

    for row_iter in optim_task_arr.rows_iter() {
        for col in row_iter {
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

    let mut costs_to_redis: String = "".to_string();
    for row_iter in optim_task_arr.rows_iter() {
        for col in row_iter {
            costs_to_redis += &(col.node_qty.to_string());
            costs_to_redis += "_";
        }
    } 
    let _: () = connection.set("initial_solve", &costs_to_redis).unwrap();

    // Perform simplex optimization
    //-------------------------------------------------------------------------------------
    simplex::simplex_optimize();
}