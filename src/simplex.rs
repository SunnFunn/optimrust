use redis::{Client,Commands};
use std::time::Instant;
use array2d::{Array2D, Error};

use crate::node;
use crate::getdata;


#[derive(Debug, Clone)]
pub enum SimplexConstraint {
    Equal(Vec<i32>, i32),
    LessThan(Vec<i32>, i32),
    GreaterThan(Vec<i32>, i32),
}

impl SimplexConstraint {
    fn get_vector(&self) -> &Vec<i32> {
        match self {
            SimplexConstraint::Equal(a, _b) => a,
            SimplexConstraint::LessThan(a, _b) => a,
            SimplexConstraint::GreaterThan(a, _b) => a,
        }
    }

    fn get_b(&self) -> i32 {
        match self {
            SimplexConstraint::Equal(_a, b) => *b,
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
    InfiniteSolution,
    NoSolution,
}

pub struct SimplexTable {
    pub objective: Vec<i32>,
    pub table: Array2D<i32>,
    pub base: Vec<usize>,
    pub vars: Vec<SimplexVar>,
}

impl SimplexTable {
    fn get_entry_var(&self) -> Option<usize> {
        let mut entry_var = None;
        let mut max_entry = -1;
        let row_size: usize = self.table.num_rows();
        let col_size: usize = self.table.num_columns();
        for i in 0..col_size {
            if i == col_size - 1 {
                continue;
            }
            if max_entry < self.table[(row_size - 1, i)] {
                max_entry = self.table[(row_size - 1, i)];
                entry_var = Some(i);
            }
        }
        entry_var
    }

    fn get_exit_var(&self, entry_var: usize) -> Option<usize> {
        let mut exit_var = None;
        // let mut min_entry = i32::MAX;
        let mut min_entry = 100000;
        let row_size: usize = self.table.num_rows();
        let col_size: usize = self.table.num_columns();

        for i in 0..row_size {
            if i == 0 || i == row_size - 1 { continue; }

            let b_i = self.table[(i, col_size - 1)];
            let pivot_i = self.table[(i, entry_var)];
            // if (pivot_i)^(b_i) > 0 { continue; }
            // else if pivot_i < 0 && b_i == 0 { continue; }
            // else if pivot_i == 0 { continue; }
            // else if pivot_i > 0 && b_i == 0 {
            //     continue;
            // }
            if pivot_i <= 0 { continue; }

            if min_entry > b_i / pivot_i {
                min_entry = b_i / pivot_i;
                exit_var = Some(self.base[i - 1]);
            }
        }
        exit_var
    }

    fn step(&mut self, entry_var: usize, exit_var: usize) {
        let exit_row_idx = self.base.iter().position(|x| *x == exit_var).unwrap() + 1;
        let pivot = self.table[(exit_row_idx, entry_var)];
        
        for col in 0..self.table.num_columns() {
            self.table[(exit_row_idx, col)] /= pivot;
        }

        for row in 0..self.table.num_rows() {
            let factor = self.table[(row, entry_var)]/pivot;
            if row == 0 || row == exit_row_idx { continue; }
            else {
                for col in 0..self.table.num_columns() {
                    let pivot_col = self.table[(exit_row_idx, col)];
                    self.table[(row, col)] -= factor*pivot_col;
                }
            }
        }
        

        self.base = self
            .base
            .iter_mut()
            .map(|x| if *x == exit_var { entry_var } else { *x })
            .collect();
        
        // let nrows = self.table.nrows();
        // for i in 0..self.table.ncols() {
        //     let mut delta: f32 = 0.0;
        //     for (j, b) in self.base.iter().enumerate() {
        //         delta += self.table[(j + 1, i)]*self.table[(0, *b)];
        //     }
        //     // println!{"delta: {}, Cj: {}", delta, table[i+1]};
        //     if i < self.table.ncols() {
        //         self.table[(nrows - 1, i)] = delta - self.table[(0, i)];
        //     }
        //     else { self.table[(nrows - 1, i)] = delta; }
        // }
    }

    pub fn solve(&mut self) -> SimplexOutput {
        let mut counter: i32 = 0;
        loop {
            counter += 1;
            if let Some(entry_var) = self.get_entry_var() {
                // println!{"{}", entry_var}
                if let Some(exit_var) = self.get_exit_var(entry_var) {
                    // println!{"{}", exit_var}
                    self.step(entry_var, exit_var);
                } else {
                    return SimplexOutput::InfiniteSolution;
                }
            } else {
                panic!("Can't continue");
            }
            // println!("Table: {:?}", self.table);
            let mut optimum = true;
            let mut unique = true;
            let nrows = self.table.num_rows();
            let ncols = self.table.num_columns();

            for i in 0..ncols {
                if i == ncols - 1 { continue; }
                let z = self.table[(nrows - 1, i)];
                optimum = optimum && z <= 0;
                if !self.base.contains(&i) && i < self.objective.len() {
                    unique = unique && z - self.objective[i] < 0;
                }
            }
            if optimum {
                let optimum = self.table[(self.table.num_rows() - 1, self.table.num_columns() - 1)];
                // for (i, var) in self.base.iter().enumerate() {
                //     if self.vars[*var].is_artificial() {
                //         if self.table.row(i + 1)[self.table.ncols() - 1] > 0.0 {
                //             /* Artificial variable might have taken slack var value */
                //             if self.vars[*var - 2].is_slack() {
                //                 if self.table.row(nrows - 1)[*var - 1] == 0.0 {
                //                     continue;
                //                 }
                //             }
                //             return SimplexOutput::NoSolution;
                //         }
                //     }
                // }
    
                if unique {
                    println!("Unique, loop counts: {}", counter);
                    return SimplexOutput::UniqueOptimum(optimum);
                } else {
                    println!("Multiple, loop counts: {}", counter);
                    return SimplexOutput::MultipleOptimum(optimum);
                }
            }
        }
    }

    pub fn get_var(&self, var: usize) -> Option<i32> {
        if var > self.objective.len() {
            return None;
        }
        for (i, v) in self.base.iter().enumerate() {
            if *v == var {
                return Some(self.table[(i + 1, self.table.num_columns() - 1)]);
            }
        }
        return Some(0);
    }

    pub fn get_target(&self) -> Option<i32> {
        return Some(self.table[(self.table.num_rows() - 1, self.table.num_columns() - 1)]);
    }
}

pub struct SimplexMinimizerBuilder {
    objective: Vec<i32>,
}

impl SimplexMinimizerBuilder {
    pub fn with(self, constraints: Vec<SimplexConstraint>) -> Result<SimplexTable, String> {
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
                _ => {
                    table.push(m_big.clone());
                    vars.push(SimplexVar::Artificial(i));
                }
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
            for (j, constraint) in constraints.iter().enumerate() {
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

        let table = Array2D::from_row_major(&table, base.len() + 2, vars.len() + 1);

        match table {
            Ok(table) => Ok(SimplexTable {
                objective: self.objective,
                table: table,
                base: base,
                vars: vars,
            }),
            Err(_) => Err(String::from("Invalid matrix")),
        }
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
    let now1 = Instant::now();

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
    let optim_task_arr = Array2D::from_row_major(&optim_task_vec, s_task_size, d_task_size).unwrap();

    // Initialize constraints
    //-------------------------------------------------------------------------------------
    let mut constraints: Vec<SimplexConstraint> = vec![];

    for i in 0..s_task_size {
        let s_qty = optim_task_arr[(i, 0)].s_qty;
        let mut s_constraint: Vec<i32> = vec![];

        for p in 0..problem_size {
            if p >= i*d_task_size && p < (i + 1)*d_task_size { s_constraint.push(1); }
            else { s_constraint.push(0); }
        }
        constraints.push(SimplexConstraint::LessThan(s_constraint, s_qty));
    }

    
    for j in 0..d_task_size {
        let d_qty = optim_task_arr[(0, j)].d_qty;
        let mut d_constraint: Vec<i32> = vec![];


        for p in 0..problem_size {
            if p < j { d_constraint.push(0); }
            else if p >= j && (p - j)%d_task_size == 0 { d_constraint.push(1); }
            else { d_constraint.push(0); }
        }
        constraints.push(SimplexConstraint::GreaterThan(d_constraint, d_qty));
    }

    let elapsed = now1.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    let program = Simplex::minimize(&costs_data).with(constraints);

    let mut simplex = program.unwrap();
    // println!("Base: {:?}", simplex.base);
    // println!("Entry: {:?}", simplex.get_entry_var());
    // let entry = simplex.get_entry_var().unwrap();
    // let exit = simplex.get_exit_var(entry).unwrap();
    // println!("Exit: {:?}", exit);

    // println!("Table to start simplex: {:.2?}", simplex.table);
    // println!("Table to start simplex:");
    // for row_iter in simplex.table.rows_iter() {
    //     for element in row_iter {
    //         print!("{} ", element);
    //     }
    //     println!();
    // }

    // simplex.step(entry, exit);

    // println!("Table to start simplex:");
    // for row_iter in simplex.table.rows_iter() {
    //     for element in row_iter {
    //         print!("{} ", element);
    //     }
    //     println!();
    // }


    let now2 = Instant::now();

    match simplex.solve() {
        SimplexOutput::UniqueOptimum(x) => println!("{}", x),
        SimplexOutput::MultipleOptimum(x) => println!("{}", x),
        _ => panic!("No solution or unbounded"),
    }

    let elapsed = now2.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    let mut assigns: i32 = 0;
    for p in 0..problem_size {
        assigns += simplex.get_var(p).unwrap();
        // println!("x{}: {}", p, simplex.get_var(p).unwrap());
    }
    println!("Total assigned amount: {}", assigns);
    println!("Total problem cost: {}", simplex.get_target().unwrap());
}