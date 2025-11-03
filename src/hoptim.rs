use highs::{Sense, ColProblem, Row};
use rand::prelude::*;
use crate::node;
use ndarray::Array;
use ndarray::Dim;
use std::time::Instant;
use std::iter::zip;


pub fn hoptim(task_array2d: &mut Array<node::Node, Dim<[usize; 2]>>,
            warmup: &mut (Vec<f64>, Vec<f64>),
            warmup_flag: &mut bool) {
    
    let s_size: usize = task_array2d.nrows();
    let d_size: usize = task_array2d.ncols();

    let mut model = ColProblem::default();
    let mut b: Vec<f64> = Vec::new();
    let mut c: Vec<f64> = Vec::new();

    // b vector generation
    for row in task_array2d.rows() { b.push(row[0].s_qty as f64); }
    for col in task_array2d.columns() { b.push(col[0].d_qty as f64); }

    // costs array generation
    for i in 0..s_size {
        for j in 0..d_size {
            c.push(task_array2d[(i, j)].node_cost as f64);
        }
    }
    let c_arr = Array::from_vec(c.clone());
    let c_costs = c_arr.into_shape_with_order((s_size, d_size)).unwrap();

    // println!("{:?}", b);
    // println!("{:?}", c_costs);
    // println!("{:?}", warmup.0);

    // constraints
    let mut rows: Vec<Row> = Vec::new();
    for constraint_idx in 0..s_size {
        let row_idx = model.add_row(0.0..b[constraint_idx]);
        rows.push(row_idx);
    }
    for constraint_idx in 0..d_size {
        let row_idx = model.add_row(b[s_size + constraint_idx]..);
        rows.push(row_idx);
    }

    // Create a variables
    for i in 0..s_size {
        for j in 0..d_size {
            model.add_integer_column(c_costs[[i, j]], 0.0.., [(rows[i], 1.), (rows[s_size + j], 1.)].into_iter());
        }
    }    

    let now = Instant::now();

    // let cols: Option<&[f64]> = Some(&[18.0, 0.0, 19.0, 0.0, 34.0, 1.0, 0.0, 0.0, 18.0]);
    // let rows: Option<&[f64]> = Some(&[37.0, 35.0, 18.0, 18.0, 34.0, 38.0]);
    // let col_duals: Option<&[f64]> = Some(&[2.0, 22.0, 9.0, 26.0, 2.0, 29.0, 13.0, 10.0, 2.0]);
    // let row_duals: Option<&[f64]> = Some(&[-3.3333, 16.6666, -10.3333, 5.3333, -14.6666, 12.3333]);
    

    // optim algo settings
    let mut optimizer = model.optimise(Sense::Minimise);
    optimizer.set_option("solver", "simplex"); // pick up solver
    optimizer.set_option("presolve", "off"); // disable the presolver
    // optimizer.set_option("time_limit", 30.0); // stop after 30 seconds
    optimizer.set_option("parallel", "on"); // use multiple cores
    optimizer.set_option("threads", 10); // solve on 4 threads

    // warm-up
    if *warmup_flag {
        let cols: Option<&[f64]> = Some(&warmup.0);
        let rows: Option<&[f64]> = Some(&b);
        let col_duals: Option<&[f64]> = Some(&c);
        let row_duals: Option<&[f64]> = Some(&warmup.1);

        optimizer.set_solution(cols, rows, col_duals, row_duals);
    }

    // solving
    let solved = optimizer.solve();
    println!("{:?}", solved.status());

    let solution = solved.get_solution();
    let mut total_cost: f64 = 0.0;
    for (qty, cost) in zip(solution.columns(), c_costs) { total_cost += qty * cost; }
    // The expected solution
    // println!("Solution vars: {:?}", solution.columns());
    // println!("Dual columns: {:?}", solution.dual_columns());
    println!("Total cost: {:?}", total_cost);
    // All the constraints are at their maximum
    // println!("{:?}", solution.rows());
    // println!("Dual rows: {:?}", solution.dual_rows());

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    let s_sum_final: f64 = solution.rows()[..s_size].iter().sum();
    let d_sum_final: f64 = solution.rows()[s_size..].iter().sum();
    let assigns: f64 = solution.columns().iter().sum();

    println!("{} {} {}", s_sum_final, d_sum_final, assigns);
}
