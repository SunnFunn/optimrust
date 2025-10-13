use std::iter::zip;
use ndarray::{Array, Slice, s};
// use ndarray::Dim;
// use ndarray::Axis;
// use ndarray::Zip;
// use ndarray::s;
// use ndarray::concatenate;
// use ndarray::parallel::prelude::*;
// use rayon::prelude::*;

// use ndarray_linalg::*;
use sprs::{CsMat, CsMatBase, TriMat};

use crate::getdata;


#[derive(Debug, Clone)]
pub enum SimplexConstraint {
    // Equal(Vec<i32>, i32),
    LessThan(Vec<i32>, i32),
    GreaterThan(Vec<i32>, i32),
}

impl SimplexConstraint {
    fn get_vector(&self) -> &Vec<i32> {
        match self {
            // SimplexConstraint::Equal(a, _b) => a,
            SimplexConstraint::LessThan(a, _b) => a,
            SimplexConstraint::GreaterThan(a, _b) => a,
        }
    }

    fn get_b(&self) -> i32 {
        match self {
            // SimplexConstraint::Equal(_a, b) => *b,
            SimplexConstraint::LessThan(_a, b) => *b,
            SimplexConstraint::GreaterThan(_a, b) => *b,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SimplexVar {
    Real,
    Slack(usize),
    NegativeSlack(usize),
    Artificial(usize),
}

impl SimplexVar {
    fn is_artificial(&self) -> bool {
        match self {
            SimplexVar::Artificial(_) => true,
            _ => false,
        }
    }

    fn is_slack(&self) -> bool {
        match self {
            SimplexVar::Slack(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SimplexOutput {
    UniqueOptimum(i32),
    MultipleOptimum(i32),
    // SubOptimum(i32),
    InfiniteSolution,
    // NoSolution,
}

pub struct SimplexTable {
    pub objective: Vec<i32>,
    // pub table: Array<i32, Dim<[usize; 2]>>,
    // pub table: CsMat<SpIndex, SpIndex, [SpIndex], [SpIndex], [i32]>,
    pub table: CsMatBase<i32, usize, Vec<usize>, Vec<usize>, Vec<i32>>,
    pub base: Vec<usize>,
    pub vars: Vec<SimplexVar>,
}

impl SimplexTable {
    fn get_entry_var(&self) -> Option<usize> {
        let mut entry_var = None;
        let mut max_entry = -1;
        // let row_size: usize = self.table.shape()[0];
        // let col_size: usize = self.table.shape()[1];
        // for i in 0..(col_size - 1) {
        //     if max_entry < self.table[(row_size - 1, i)] {
        //         max_entry = self.table[(row_size - 1, i)];
        //         entry_var = Some(i);
        //     }
        // }

        let nrows = self.table.rows();
        let ncols = self.table.cols();

        for ele in self.table.iter() {
            if ele.1.0 == nrows - 1 && ele.1.1 != ncols - 1{
                if max_entry < *ele.0 {
                    max_entry = *ele.0;
                    entry_var = Some(ele.1.1);
                }
            }
        }
        entry_var
    }

    fn get_exit_var(&self, entry_var: usize) -> Option<usize> {
        let mut exit_var = Some(1);
        // let mut min_entry = i32::MAX;
        let mut min_entry = 100000;
        // let row_size: usize = self.table.shape()[0];
        // let col_size: usize = self.table.shape()[1];
        let nrows = self.table.rows();
        let ncols = self.table.cols();

        for row_idx in 0..(nrows - 2) {
            let row_view = self.table
                            .outer_view(row_idx)
                            .unwrap();
            let idxs = row_view.indices();
            if idxs.contains(&entry_var) {
                let b_i = row_view[idxs[idxs.len() - 1]];
                let pivot_i = row_view[entry_var];
                if pivot_i <= 0 { continue; }

                if min_entry > b_i / pivot_i {
                    min_entry = b_i / pivot_i;
                    exit_var = Some(self.base[row_idx]);
                }
            }
        }

        // for i in 1..(row_size - 1) {
        //     let b_i = self.table[(i, col_size - 1)];
        //     let pivot_i = self.table[(i, entry_var)];
        //     if pivot_i <= 0 { continue; }

        //     if min_entry > b_i / pivot_i {
        //         min_entry = b_i / pivot_i;
        //         exit_var = Some(self.base[i - 1]);
        //     }
        // }
        exit_var
    }

    fn step(&mut self, entry_var: usize, exit_var: usize) {
        let exit_row_idx = self.base.iter().position(|x| *x == exit_var).unwrap();

        //-------------------------------------- CST -----------------------------------------

        let pivot_range = &self.table.indptr().outer_inds_sz(exit_row_idx);
        let pivot_row_idxs = &self.table.indices()[pivot_range.start..pivot_range.end];
        let pivot_row_data = &self.table.data()[pivot_range.start..pivot_range.end];

        let p_index = pivot_row_idxs.iter().position(|x| *x == entry_var).unwrap();
        let pivot = pivot_row_data[p_index];

        // if pivot != 1 {
        //     self.table.outer_view(exit_row_idx).unwrap().map(|x| x/pivot);
        // }

        let nrows = self.table.rows();
        let mut table_copy = self.table.clone();

        for i in 0..(nrows) {
            let row_range = self.table.indptr().outer_inds_sz(i);
            let row_idxs = &self.table.indices()[row_range.start..row_range.end];
            let row_data = &self.table.data()[row_range.start..row_range.end];

            if i == exit_row_idx { continue }
            else {
                if row_idxs.contains(&entry_var) {
                    println!("{:?}  {}", row_idxs, i);

                    let r_index = row_idxs.iter().position(|x| *x == entry_var).unwrap();
                    let factor = row_data[r_index] / pivot;

                    println!("{:?}  {}", r_index, factor);
                    println!("{:?}", &pivot_row_idxs);

                    for col_idx in pivot_row_idxs { 
                        let pivot_index = pivot_row_idxs.iter().position(|x| *x == *col_idx).unwrap();
                        
                        let mut new_ele = 0;
                        if row_idxs.contains(&col_idx){
                            let row_index = row_idxs.iter().position(|x| *x == *col_idx).unwrap();
                            new_ele = row_data[row_index] - pivot_row_data[pivot_index] * factor;
                        }
                        else{
                            new_ele = - pivot_row_data[pivot_index] * factor;
                        }

                        table_copy.insert(i, *col_idx, new_ele);
                    }
                }
            };

        }

        self.table = table_copy;
        
        println!("{:?}", self.table.to_dense());
        // println!("{:?}", pivot);
        // println!("{:?}", pivot_row_idxs);
        // println!("{:?}", pivot_row_data);

        // -------------------------------------- Simple Cycling -----------------------------------------

        // for row in 1..self.table.nrows() {
        //     if self.table[(row, entry_var)] != 0 && row != exit_row_idx {

        //         let factor = self.table[(row, entry_var)]/pivot;
        //         let exit_row = self.table.row(exit_row_idx).to_owned();
        //         let mut row_mut = self.table.row_mut(row);

        //         // Zip::from(&mut row_mut)
        //         // .and(&exit_row)
        //         // .par_for_each(|x, &y| {
        //         //     *x -= factor * y;
        //         // });

        //         for col in 0..self.table.shape()[1] {
        //             if self.table[(exit_row_idx, col)] != 0 {
        //                 self.table[(row, col)] -= factor*self.table[(exit_row_idx, col)];
        //             }
        //         }
        //     }
        // }

        self.base = self
            .base
            .iter_mut()
            .map(|x| if *x == exit_var { entry_var } else { *x })
            .collect();
    }

//     pub fn solve(&mut self) -> SimplexOutput {
//         let mut counter: i32 = 0;
//         loop {
//             counter += 1;

//             if let Some(entry_var) = self.get_entry_var() {
//                 if let Some(exit_var) = self.get_exit_var(entry_var) {
//                     self.step(entry_var, exit_var);
//                 } else {
//                     return SimplexOutput::InfiniteSolution;
//                 }
//             } else {
//                 panic!("Can't continue");
//             }
//             // println!("All elements:");
//             // for row_iter in self.table.rows_iter() {
//             //     for element in row_iter {
//             //         print!("{} ", element);
//             //     }
//             //     println!();
//             // }
//             let mut optimum = true;
//             let mut unique = true;
//             let nrows = self.table.shape()[0];
//             let ncols = self.table.shape()[1];

//             for i in 0..(ncols - 1) {
//                 // if i == ncols - 1 { continue; }
//                 let z = self.table[(nrows - 1, i)];
//                 optimum = optimum && z <= 0;
//                 if !self.base.contains(&i) && i < self.objective.len() {
//                     unique = unique && z - self.objective[i] < 0;
//                 }
//             }
//             if optimum {
//                 let optimum = self.table[(self.table.shape()[0] - 1, self.table.shape()[1] - 1)];
//                 // for (i, var) in self.base.iter().enumerate() {
//                 //     if self.vars[*var].is_artificial() {
//                 //         if self.table.row(i + 1)[self.table.ncols() - 1] > 0.0 {
//                 //             /* Artificial variable might have taken slack var value */
//                 //             if self.vars[*var - 2].is_slack() {
//                 //                 if self.table.row(nrows - 1)[*var - 1] == 0.0 {
//                 //                     continue;
//                 //                 }
//                 //             }
//                 //             return SimplexOutput::NoSolution;
//                 //         }
//                 //     }
//                 // }
    
//                 if unique {
//                     println!("Unique, loop counts: {}", counter);
//                     return SimplexOutput::UniqueOptimum(optimum);
//                 } else {
//                     println!("Multiple, loop counts: {}", counter);
//                     return SimplexOutput::MultipleOptimum(optimum);
//                 }
//             }
//             // if counter > 150 {
//             //     let sub_optimum = self.table[(self.table.num_rows() - 1, self.table.num_columns() - 1)];
//             //     return SimplexOutput::SubOptimum(sub_optimum);
//             // }
//         }
//     }

//     pub fn get_var(&self, var: usize) -> Option<i32> {
//         if var > self.objective.len() {
//             return None;
//         }
//         for (i, v) in self.base.iter().enumerate() {
//             if *v == var {
//                 return Some(self.table[(i + 1, self.table.shape()[1] - 1)]);
//             }
//         }
//         return Some(0);
//     }

//     pub fn get_target(&self) -> Option<i32> {
//         return Some(self.table[(self.table.shape()[0] - 1, self.table.shape()[1] - 1)]);
//     }
}

pub struct SimplexMinimizerBuilder {
    objective: Vec<i32>,
}

impl SimplexMinimizerBuilder {
    pub fn with(self, constraints: Vec<SimplexConstraint>) -> SimplexTable {
        let mut table = Vec::new();
        let mut vars = Vec::new();
        let m_big: i32 = 1000;
        
        for var in self.objective.iter() {
            table.push(*var);
            vars.push(SimplexVar::Real);
        }
        for (i, constraint) in constraints.iter().enumerate() {
            match constraint {
                SimplexConstraint::LessThan(_, _) => {
                    table.push(0);
                    vars.push(SimplexVar::Slack(i));
                }
                SimplexConstraint::GreaterThan(_, _) => {
                    table.push(0);
                    vars.push(SimplexVar::NegativeSlack(i));
                    table.push(m_big.clone());
                    vars.push(SimplexVar::Artificial(i));
                }
                // _ => {
                //     table.push(m_big.clone());
                //     vars.push(SimplexVar::Artificial(i));
                // }
            }
        }
        table.push(0);

        for (i, constraint) in constraints.iter().enumerate() {
            for a in constraint.get_vector() {
                table.push(*a);
            }
            for var in vars.iter() {
                match var {
                    SimplexVar::Slack(j) => {
                        if *j == i {
                            table.push(1);
                        } else {
                            table.push(0);
                        }
                    }
                    SimplexVar::NegativeSlack(j) => {
                        if *j == i {
                            table.push(-1);
                        } else {
                            table.push(0);
                        }
                    }
                    SimplexVar::Artificial(j) => {
                        if *j == i {
                            table.push(1);
                        } else {
                            table.push(0);
                        }
                    }
                    _ => {}
                }
            }
            table.push(constraint.get_b());
        }

        let base: Vec<usize> = vars
            .iter()
            .enumerate()
            .filter_map(|(i, x)| if x.is_artificial() || x.is_slack() { Some(i) } else { None })
            .collect();
        
        for i in 0..vars.len() {
            let mut delta: i32 = 0;
            for (j, _constraint) in constraints.iter().enumerate() {
            delta += table[(j + 1)*(vars.len() + 1) + i]*table[base[j]];
            }
            delta = delta - table[i];
            table.push(delta);
        }

        let mut target: i32 = 0;
        for (i, constraint) in constraints.iter().enumerate() {
            target += constraint.get_b()*table[base[i]];
        }
        table.push(target);

        let table = Array::from_vec(table.clone())
                                .clone()
                                .into_shape_with_order((base.len() + 2, vars.len() + 1))
                                .unwrap();
        
        let nrows = table.shape()[0] - 1;
        let ncols = table.shape()[1];
        let mut table_trimat = TriMat::new((nrows, ncols));
        for (i, ele) in table.slice(s![1.., ..]).iter().enumerate() {
            if *ele != 0 { table_trimat.add_triplet((i - i%ncols)/ncols, i%ncols, *ele)};
        }
        let mut table_cst: CsMat<_> = table_trimat.to_csr();

        SimplexTable {
            objective: self.objective,
            table: table_cst,
            base: base,
            vars: vars,
        }

        // match table_cst {
        //     Ok(table_cst) => Ok(SimplexTable {
        //         objective: self.objective,
        //         table: table_cst,
        //         base: base,
        //         vars: vars,
        //     }),
        //     Err(_) => Err(String::from("Invalid matrix")),
        // }
    }
}

pub struct Simplex;

impl Simplex {
    pub fn minimize(objective: &Vec<i32>) -> SimplexMinimizerBuilder {
        SimplexMinimizerBuilder {
            objective: objective.clone(),
        }
    }
}

pub fn simplex_optimize () {
    // get task vector & task sizes
    //-------------------------------------------------------------------------------------
    let optim_data = getdata::get().unwrap();
    let optim_task_vec = optim_data.0;
    let s_task_size = optim_data.1;
    let d_task_size = optim_data.2;
    let problem_size = s_task_size*d_task_size;

    let mut costs_data = Vec::<i32>::with_capacity(problem_size);
    for i in 0..problem_size {
        costs_data.push(optim_task_vec[i].node_cost)
    }

    // Initialize task matrix as 2D array from task vector
    //-------------------------------------------------------------------------------------
    let mut optim_task_array2d = Array::from_vec(optim_task_vec.clone())
                                .clone()
                                .into_shape_with_order((s_task_size, d_task_size))
                                .unwrap();

    // Initialize constraints
    //-------------------------------------------------------------------------------------
    let mut constraints: Vec<SimplexConstraint> = vec![];

    for i in 0..s_task_size {
        let s_qty = optim_task_array2d[(i, 0)].s_qty;
        let mut s_constraint: Vec<i32> = vec![];

        for p in 0..problem_size {
            if p >= i*d_task_size && p < (i + 1)*d_task_size { s_constraint.push(1); }
            else { s_constraint.push(0); }
        }
        constraints.push(SimplexConstraint::LessThan(s_constraint, s_qty));
    }
    
    for j in 0..d_task_size {
        let d_qty = optim_task_array2d[(0, j)].d_qty;
        let mut d_constraint: Vec<i32> = vec![];


        for p in 0..problem_size {
            if p < j { d_constraint.push(0); }
            else if p >= j && (p - j)%d_task_size == 0 { d_constraint.push(1); }
            else { d_constraint.push(0); }
        }
        constraints.push(SimplexConstraint::GreaterThan(d_constraint, d_qty));
    }

    let mut simplex = Simplex::minimize(&costs_data).with(constraints);

    // let mut simplex = program.unwrap();
    println!("{:?}", simplex.table.to_dense());

    let mut evar = simplex.get_entry_var().unwrap();
    let mut exitvar = simplex.get_exit_var(evar);
    println!("{:?}  {:?}", evar, exitvar.unwrap());
    simplex.step(evar, exitvar.unwrap());

    evar = simplex.get_entry_var().unwrap();
    exitvar = simplex.get_exit_var(evar);
    println!("{:?}  {:?}", evar, exitvar.unwrap());
    simplex.step(evar, exitvar.unwrap());

    evar = simplex.get_entry_var().unwrap();
    exitvar = simplex.get_exit_var(evar);
    println!("{:?}  {:?}", evar, exitvar.unwrap());
    simplex.step(evar, exitvar.unwrap());

    evar = simplex.get_entry_var().unwrap();
    exitvar = simplex.get_exit_var(evar);
    println!("{:?}  {:?}", evar, exitvar.unwrap());
    simplex.step(evar, exitvar.unwrap());

    // match simplex.solve() {
    //     SimplexOutput::UniqueOptimum(x) => println!("{}", x),
    //     SimplexOutput::MultipleOptimum(x) => println!("{}", x),
    //     _ => panic!("No solution or unbounded"),
    // }

    // let mut assigns: i32 = 0;
    // for p in 0..problem_size {
    //     assigns += simplex.get_var(p).unwrap();
    //     // println!("x{}: {}", p, simplex.get_var(p).unwrap());
    // }
    // println!("Total assigned amount: {}", assigns);
    // println!("Total problem cost: {}", simplex.get_target().unwrap());
}