// use lapack::*;
// use blas::*;
use ndarray::{Array,Dim};

use crate::node;

pub fn warmup(task_array2d: &mut Array<node::Node, Dim<[usize; 2]>>) -> (Vec<f64>, Vec<f64>) {
    let s_size: usize = task_array2d.nrows();
    let d_size: usize = task_array2d.ncols();

    // suboptimal plan vector generation
    let mut x: Vec<f64> = Vec::new();
    for i in 0..s_size {
        for j in 0..d_size {
            x.push(task_array2d[(i, j)].node_qty as f64);
        }
    }

    let mut vars_vec: Vec<(usize, usize)> = Vec::new();
    for i in 0..s_size {
        for j in 0..d_size {
            if task_array2d[(i, j)].node_qty != 0 {
                vars_vec.push((i, j));
            }
        }
    }

    const INF: f64 = 1000000.0;
    let mut b: Vec<f64> = vec![INF; s_size + d_size];

    let start_value: f64 = 1.;
    b[vars_vec[0].0] = start_value;
    b[vars_vec[0].1 + s_size] = (task_array2d[vars_vec[0]].node_cost as f64) - start_value;

    let mut check: bool = true;
    while check {
        for var in vars_vec[1..].into_iter() {
            if b[var.0] != INF && b[var.1 + s_size] == INF {
                b[var.1 + s_size] = (task_array2d[*var].node_cost as f64) - b[var.0];
            }
            else if b[var.0] == INF && b[var.1 + s_size] != INF {
                b[var.0] = (task_array2d[*var].node_cost as f64) - b[var.1 + s_size];
            }
        }
        check = b.contains(&INF);
        if check {
            for var in vars_vec[1..].into_iter() {
                if b[var.0] == INF && b[var.1 + s_size] == INF {
                    b[var.0] = start_value;
                    b[var.1 + s_size] = (task_array2d[*var].node_cost as f64) - start_value;
                }
            }
        }
    }
    (x, b)

    // let mut vars_vec: Vec<(usize, usize)> = Vec::new();
    // for i in 0..s_size {
    //     for j in 0..d_size {
    //         if task_array2d[(i, j)].node_qty != 0 {
    //             vars_vec.push((task_array2d[(i, j)].s_node_id, task_array2d[(i, j)].d_node_id));
    //             b.push(task_array2d[(i, j)].node_cost as f32);
    //         }
    //     }
    // }

    // let nvars = vars_vec.len();
    // let mut a: Vec<f32> = vec![0.0; nvars*(s_size + d_size)];
    // for (i, node) in vars_vec.iter().enumerate() {
    //     a[node.0*nvars + i] = 1.0;
    //     a[(node.1 + d_size)*nvars + i] = 1.0;
    // }

    // let a_arr = Array::from_vec(a.clone());
    // let a_arr = a_arr.into_shape_with_order((s_size + d_size, nvars)).unwrap();

    // println!("{:?}", a_arr);
    // println!("{:?}", b);
    // println!("{:?}", &vars_vec[1..]);

    // let trans: u8 = b'N';
    // let nb: i32 = 32;
    // let m: i32 = nvars as i32;
    // let n: i32 = (s_size + d_size) as i32;
    // let nrhs: i32 = 1;
    
    // // // Some(&[18.0, 0.0, 19.0, 0.0, 34.0, 1.0, 0.0, 0.0, 18.0]);
    // // let mut a = vec![1., 1., 0., 0., 0.,
    // //                 0., 0., 1., 1., 0.,
    // //                 0., 0., 0., 0., 1.,
    // //                 1., 0., 0., 0., 0.,
    // //                 0., 0., 1., 0., 0.,
    // //                 0., 1., 0., 1., 1.];

    // let lda: i32 = m;
    // let ldb: i32 = n;
    // let mut work = vec![0.0; (n + n*nb) as usize];
    // let lwork = n + m*nb;
    // let mut info = 0;

    // unsafe {
    //     sgels(trans, m, n, nrhs, &mut a, lda, &mut b, ldb, &mut work, lwork, &mut info)
    // }

    // println!("{:?}", info);

    // println!("{:?}", b);
}