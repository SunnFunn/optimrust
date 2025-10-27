use highs::{Sense, ColProblem, Row};
use rand::prelude::*;
use ndarray::Array;
use std::time::Instant;


fn main() {
    let s_size: usize = 300;
    let d_size: usize = 300;

    let mut rng = rand::rng();

    let mut model = ColProblem::default();
    let mut b = Vec::new();
    let mut c = Vec::new();

    // b vector generation
    for _ in 0..(s_size + d_size) {
        let ele: i32 = rng.random_range(10..=100);
        b.push(ele as f64);
    }
    let s_sum: f64 = b[..s_size].iter().sum();
    let d_sum: f64 = b[s_size..].iter().sum();
    if d_sum > s_sum { b[s_size - 1] += d_sum - s_sum; }
    else { b[s_size +d_size - 1] += s_sum - d_sum; }

    // c array generation
    for _ in 0..s_size {
        for _ in 0..d_size {
            let ele: i32 = rng.random_range(100..=1000);
            c.push(ele as f64);
        }
    }
    let c_arr = Array::from_vec(c);
    let c_costs = c_arr.into_shape_with_order((s_size, d_size)).unwrap();

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

    let c: Vec<Vec<f64>> = vec![vec![10., 2.], vec![24., 15.]];
    // Create a variables
    for i in 0..s_size {
        for j in 0..d_size {
            model.add_column(c_costs[[i, j]], 0.0.., [(rows[i], 1.), (rows[s_size + j], 1.)].into_iter());
        }
    }    

    let now = Instant::now();

    let mut optimizer = model.optimise(Sense::Minimise);
    optimizer.set_option("solver", "simplex"); // pick up solver
    // optimizer.set_option("presolve", "off"); // disable the presolver
    // optimizer.set_option("time_limit", 30.0); // stop after 30 seconds
    optimizer.set_option("parallel", "on"); // use multiple cores
    optimizer.set_option("threads", 6); // solve on 4 threads
    let solved = optimizer.solve();
    println!("{:?}", solved.status());

    let solution = solved.get_solution();
    // The expected solution is x=0  y=6  z=0.5
    // println!("{:?}", solution.columns());
    // All the constraints are at their maximum
    // println!("{:?}", solution.rows());

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    let s_sum_final: f64 = solution.rows()[..s_size].iter().sum();
    let d_sum_final: f64 = solution.rows()[s_size..].iter().sum();
    let assigns: f64 = solution.columns().iter().sum();

    println!("{} {} {}", s_sum_final, d_sum_final, assigns);
}
