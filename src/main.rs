#[allow(dead_code)]
use redis::Client;
use std::time::Instant;
use ndarray::Array;
// use ndarray::ShapeBuilder;

mod node;
mod greedy;
mod dpreference;
mod simplex;
mod getdata;
mod hoptim;
mod warmup;


fn main () {
    let now = Instant::now();

    // // rset connection to redis DB
    // //-------------------------------------------------------------------------------------
    // let client = Client::open("redis://:alext@127.0.0.1:6379/").unwrap();
    // let mut connection = client.get_connection().unwrap();

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

    let mut optim_task_array2d_copy = optim_task_array2d.clone();

    // Perform greedy optimization
    //-------------------------------------------------------------------------------------
    greedy::greedy(&mut optim_task_vec, &mut optim_task_array2d);

    // // Perform double preference optimization
    // //-------------------------------------------------------------------------------------
    // dpreference::dpreference(&mut optim_task_array2d_copy);

    // Perform highs optimization
    //-------------------------------------------------------------------------------------
    let mut warmup_flag: bool = false;
    let mut warmup: (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());

    if warmup_flag {
        warmup = warmup::warmup(&mut optim_task_array2d);
        // println!("{:?}", warmup.1);
    }
    hoptim::hoptim(&mut optim_task_array2d_copy, &mut warmup, &mut warmup_flag);

    
    // Printout results
    //-------------------------------------------------------------------------------------
    let mut s_total = 0;
    let mut d_total = 0;
    let mut total_assignment_qty = 0;
    let mut total_cost = 0;

    for row in optim_task_array2d.rows() {
        s_total += row[0].s_qty;
    }

    for col in optim_task_array2d.columns() {
        d_total += col[0].d_qty;
    }

    for row in optim_task_array2d.rows() {
        for col in row.iter() {
            total_assignment_qty += col.node_qty;
            total_cost += col.node_qty*(col.node_cost as i32);
        }
    }
    
    println!("Total assignment qty: {}", total_assignment_qty);
    println!("Total problem cost: {}", total_cost);
    println!("Total left supply: {}", s_total);
    println!("Total left demand: {}", d_total);
    // println!("{:?}", optim_task_array2d);

    // // Printout results dpref
    // //-------------------------------------------------------------------------------------
    // let mut s_total = 0;
    // let mut d_total = 0;
    // let mut total_assignment_qty = 0;
    // let mut total_cost = 0;

    // for row in optim_task_array2d_copy.rows() {
    //     s_total += row[0].s_qty;
    // }

    // for col in optim_task_array2d_copy.columns() {
    //     d_total += col[0].d_qty;
    // }

    // for row in optim_task_array2d_copy.rows() {
    //     for col in row.iter() {
    //         total_assignment_qty += col.node_qty;
    //         total_cost += col.node_qty*(col.node_cost as i32);
    //     }
    // }
    
    // println!("Total assignment qty: {}", total_assignment_qty);
    // println!("Total problem cost: {}", total_cost);
    // println!("Total left supply: {}", s_total);
    // println!("Total left demand: {}", d_total);


    // // Perform simplex optimization
    // //-------------------------------------------------------------------------------------
    // simplex::simplex_optimize();

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}