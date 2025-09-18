// // use std::iter::zip;
// use ndarray::*;
// use redis::{Client,Commands};
// use std::time::Instant;
// // use clap::{App,Arg};

// #[cfg(test)]
// mod test;


// #[derive(Debug, Clone)]
// pub enum SimplexConstraint {
//     Equal(Vec<f64>, f64),
//     LessThan(Vec<f64>, f64),
//     GreaterThan(Vec<f64>, f64),
// }

// impl SimplexConstraint {
//     fn get_vector(&self) -> &Vec<f64> {
//         match self {
//             SimplexConstraint::Equal(a, _b) => a,
//             SimplexConstraint::LessThan(a, _b) => a,
//             SimplexConstraint::GreaterThan(a, _b) => a,
//         }
//     }

//     fn get_b(&self) -> f64 {
//         match self {
//             SimplexConstraint::Equal(_a, b) => *b,
//             SimplexConstraint::LessThan(_a, b) => *b,
//             SimplexConstraint::GreaterThan(_a, b) => *b,
//         }
//     }
// }

// #[derive(Clone, Debug, PartialEq)]
// pub enum SimplexVar {
//     Real,
//     Slack(usize),
//     NegativeSlack(usize),
//     Artificial(usize),
// }

// impl SimplexVar {
//     fn is_artificial(&self) -> bool {
//         match self {
//             SimplexVar::Artificial(_) => true,
//             _ => false,
//         }
//     }

//     fn is_slack(&self) -> bool {
//         match self {
//             SimplexVar::Slack(_) => true,
//             _ => false,
//         }
//     }
// }

// #[derive(Debug, PartialEq)]
// pub enum SimplexOutput {
//     UniqueOptimum(f64),
//     MultipleOptimum(f64),
//     InfiniteSolution,
//     NoSolution,
// }

// pub struct SimplexTable {
//     pub objective: Vec<f64>,
//     pub table: Array2<f64>,
//     pub base: Vec<usize>,
//     pub vars: Vec<SimplexVar>,
// }

// impl SimplexTable {
//     fn get_entry_var(&self) -> Option<usize> {
//         let mut entry_var = None;
//         let mut max_entry = -1.0;
//         let x_size: usize = self.table.len_of(Axis(0));
//         for (i, z) in self.table.row(x_size - 1).iter().enumerate() {
//             if i == 0 || i == self.table.ncols() - 1 {
//                 continue;
//             }
//             if max_entry < *z {
//                 max_entry = *z;
//                 entry_var = Some(i);
//             }
//         }
//         entry_var
//     }

//     fn get_exit_var(&self, entry_var: usize) -> Option<usize> {
//         let mut exit_var = None;
//         let mut min_entry = f64::MAX;
//         let b = self.table.column(self.table.ncols() - 1);
//         for (i, z) in self.table.column(entry_var).iter().enumerate() {
//             if i == 0 || i == self.table.nrows() -1 {
//                 continue;
//             }
//             if *z <= 0.0 {
//                 continue;
//             }
//             if min_entry > b[i] / z {
//                 min_entry = b[i] / z;
//                 exit_var = Some(self.base[i - 1]);
//             }
//         }
//         exit_var
//     }

//     fn step(&mut self, entry_var: usize, exit_var: usize) {
//         let exit_row_idx = self.base.iter().position(|x| *x == exit_var).unwrap() + 1;
//         let pivot = self.table.row(exit_row_idx)[entry_var];
//         {
//             let mut row = self.table.row_mut(exit_row_idx);
//             row /= pivot;
//         }
    
//         let start = Instant:: now();
//         let exit_row = self.table.row(exit_row_idx).to_owned();
//         for i in 1..self.table.nrows() {
//             if i == exit_row_idx { continue; }
            
//             let col_size: usize = self.table.len_of(Axis(1));
//             // let mut exit_row = self.table.row(exit_row_idx).to_owned();
//             let mut row = self.table.row_mut(i);
//             let factor: Vec<f64> = [row[entry_var] / -1.0; col_size];
//             let exit_row_factored = factor*exit_row;
//             // row /= factor;
//             // row = (row + &exit_row)*factor;

//             // exit_row *= factor;
//             row += exit_row_factored;
//             // row *= factor;
            
//         }

//         // for i in 1..self.table.nrows() {
//         //     if i == exit_row_idx {
//         //         continue;
//         //     }
//         //     let factor = self.table.row(i)[entry_var] / -1.0;
//         //     for j in 0..self.table.ncols() {
//         //         self.table.row_mut(i)[j] += factor*exit_row[j];
//         //     }
//         //     // // let mut exit_row = self.table.row(exit_row_idx).to_owned();
//         //     // let mut row = self.table.row_mut(i);
//         //     // let factor = row[entry_var] / -1.0;
//         //     // // exit_row *= factor;
//         //     // row += &exit_row;
//         // }
//         let end = start.elapsed();
//         println!("Elapsed loops time: {:.2?}", end);
        
//         self.base = self
//             .base
//             .iter_mut()
//             .map(|x| if *x == exit_var { entry_var } else { *x })
//             .collect();
        
//         // let mut target: f64 = 0.0;
//         // for i in 1..(self.table.nrows() - 1) {
//         //     let c_row = self.table.row(self.table.nrows() - 1).to_owned();
//         //     target += self.table.row(i)[self.table.ncols() - 1]*c_row[self.base[i - 1]];
//         // }
//         // let mut c_row = self.table.row_mut(self.table.nrows() - 1);
//         // c_row[0] = target;
//         // println!{"{}", target}
//     }

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
//             let mut optimum = true;
//             let mut unique = true;
//             let nrows = self.table.len_of(Axis(0));
//             for (i, &z) in self.table.row(nrows - 1).iter().enumerate() {
//                 if i == self.table.ncols() - 1 { continue; }
//                 optimum = optimum && z <= 0.0;
//                 if !self.base.contains(&i) && i < self.objective.len() {
//                     unique = unique && z - self.objective[i] < 0.0;
//                 }
//             }
//             if optimum {
//                 let optimum = self.table.row(0)[self.table.ncols() - 1];
//                 for (i, var) in self.base.iter().enumerate() {
//                     if self.vars[*var - 1].is_artificial() {
//                         if self.table.row(i + 1)[self.table.ncols() - 1] > 0.0 {
//                             /* Artificial variable might have taken slack var value */
//                             if self.vars[*var - 2].is_slack() {
//                                 if self.table.row(0)[*var - 1] == 0.0 {
//                                     continue;
//                                 }
//                             }
//                             return SimplexOutput::NoSolution;
//                         }
//                     }
//                 }
//                 // return SimplexOutput::UniqueOptimum(optimum);
//                 if unique {
//                     println!("loop counts: {}", counter);
//                     return SimplexOutput::UniqueOptimum(optimum);
//                 } else {
//                     println!("loop counts: {}", counter);
//                     return SimplexOutput::MultipleOptimum(optimum);
//                 }
//             }
//         }
//     }

//     pub fn get_var(&self, var: usize) -> Option<f64> {
//         if var > self.objective.len() {
//             return None;
//         }
//         for (i, v) in self.base.iter().enumerate() {
//             if *v == var {
//                 return Some(self.table.row(i + 1)[self.table.ncols() - 1]);
//             }
//         }
//         return Some(0.0);
//     }

//     pub fn get_target(&self) -> Option<f64> {
//         return Some(self.table.row(self.table.nrows() - 1)[self.table.ncols() - 1]);
//     }
// }

// pub struct SimplexMinimizerBuilder {
//     objective: Vec<f64>,
// }

// impl SimplexMinimizerBuilder {
//     pub fn with(self, constraints: Vec<SimplexConstraint>) -> Result<SimplexTable, String> {
//         let mut table = Vec::new();
//         let mut vars = Vec::new();
//         let m_big: f64 = 1000000.0;
//         table.push(1.0);
//         for var in self.objective.iter() {
//             // table.push(var * -1.0);
//             table.push(*var);
//             vars.push(SimplexVar::Real);
//         }
//         for (i, constraint) in constraints.iter().enumerate() {
//             match constraint {
//                 SimplexConstraint::LessThan(_, _) => {
//                     table.push(0.0);
//                     vars.push(SimplexVar::Slack(i));
//                 }
//                 SimplexConstraint::GreaterThan(_, _) => {
//                     table.push(0.0);
//                     vars.push(SimplexVar::NegativeSlack(i));
//                     // table.push(f64::MIN);
//                     table.push(m_big.clone());
//                     vars.push(SimplexVar::Artificial(i));
//                 }
//                 _ => {
//                     // table.push(f64::MIN);
//                     table.push(m_big.clone());
//                     vars.push(SimplexVar::Artificial(i));
//                 }
//             }
//             // table.push(f64::MIN);
//             // vars.push(SimplexVar::Artificial(i));
//         }
//         table.push(0.0);

//         for (i, constraint) in constraints.iter().enumerate() {
//             table.push(0.0);
//             for a in constraint.get_vector() {
//                 table.push(*a);
//             }
//             for var in vars.iter() {
//                 match var {
//                     SimplexVar::Slack(j) => {
//                         if *j == i {
//                             table.push(1.0);
//                         } else {
//                             table.push(0.0);
//                         }
//                     }
//                     SimplexVar::NegativeSlack(j) => {
//                         if *j == i {
//                             table.push(-1.0);
//                         } else {
//                             table.push(0.0);
//                         }
//                     }
//                     SimplexVar::Artificial(j) => {
//                         if *j == i {
//                             table.push(1.0);
//                         } else {
//                             table.push(0.0);
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//             table.push(constraint.get_b());
//         }

//         let base: Vec<usize> = vars
//             .iter()
//             .enumerate()
//             .filter_map(|(i, x)| if x.is_artificial() || x.is_slack() { Some(i + 1) } else { None })
//             .collect();
        
//         table.push(0.0);
//         for i in 0..vars.len() {
//             let mut delta: f64 = 0.0;
//             for (j, b) in base.iter().enumerate() {
//                 delta += table[(j + 1)*(vars.len() + 2) + i + 1]*table[*b];
//             }
//             // println!{"delta: {}, Cj: {}", delta, table[i+1]};
//             delta = delta - table[i+1];
//             table.push(delta);
//         }

//         let mut target: f64 = 0.0;
//         // for it in zip(constraints.clone, base.clone()) {
//         //     target += it.0.get_b()*table[it.1];
//         // }
//         for (i, constraint) in constraints.iter().enumerate() {
//             target += constraint.get_b()*table[base[i]];
//         }
//         println!{"{}", target};
//         table.push(target);

//         let table = Array2::from_shape_vec((base.len() + 2, vars.len() + 2), table);

//         match table {
//             Ok(table) => Ok(SimplexTable {
//                 objective: self.objective,
//                 table: table,
//                 base: base,
//                 vars: vars,
//             }),
//             Err(_) => Err(String::from("Invalid matrix")),
//         }
//     }
// }


// pub struct Simplex;

// impl Simplex {
//     pub fn minimize(objective: &Vec<f64>) -> SimplexMinimizerBuilder {
//         SimplexMinimizerBuilder {
//             objective: objective.clone(),
//         }
//     }
// }

// fn main(){
//     let client = Client::open("redis://:alext@127.0.0.1:6379/").unwrap();
//     let mut connection = client.get_connection().unwrap();

//     // let _: () = connection.set("name", name).unwrap();

//     let supply_str: String = connection.get("supply").unwrap();
//     let demand_str: String = connection.get("demand").unwrap();
//     let costs_str: String = connection.get("costs").unwrap();

//     let supply_vec: Vec<&str> = supply_str.split(" ").collect();
//     let demand_vec: Vec<&str> = demand_str.split(" ").collect();
//     let costs_vec: Vec<&str> = costs_str.split(" ").collect();

//     #[derive(Debug)]
//     struct Costs {
//         s_node_ids: Vec<i16>,
//         d_node_ids: Vec<i16>,
//         costs: Vec<f64>,
//     }

//     impl Costs {
//         fn new() -> Costs {
//             Costs {
//                 s_node_ids: Vec::new(),
//                 d_node_ids: Vec::new(),
//                 costs: Vec::new(),
//             }
//         }

//         fn add_data(&mut self, s_id: i16, d_id: i16, cost: f64) {
//             self.s_node_ids.push(s_id);
//             self.d_node_ids.push(d_id);
//             self.costs.push(cost);
//         }
//     }

//     let now1 = Instant::now();

//     let mut costs_data = Costs::new();
//     for cost in costs_vec.iter() {
//         let c_vec: Vec<&str> = cost.split("_").collect();
//         costs_data.add_data(c_vec[0].parse().unwrap(),
//                             c_vec[1].parse().unwrap(),
//                             c_vec[2].parse().unwrap())
//     }

//     let s_size = supply_vec.len();
//     let d_size = demand_vec.len();
//     let problem_size = s_size*d_size;

//     let mut constraints: Vec<SimplexConstraint> = vec![];
    
//     // let mut d_constraints: Vec<i8> = vec![];

//     for (i, s) in supply_vec.iter().enumerate() {
//         let s_vec: Vec<&str> = s.split("_").collect();
//         let s_qty = s_vec[1].parse().unwrap();
        
//         let mut s_constraint: Vec<f64> = vec![];
//         for p in 0..problem_size {
//             if p >= i*d_size && p < (i + 1)*d_size { s_constraint.push(1.0); }
//             else { s_constraint.push(0.0); }
//         }
//         constraints.push(SimplexConstraint::LessThan(s_constraint, s_qty));
//     }
//     for (j, d) in demand_vec.iter().enumerate() {
//         let d_vec: Vec<&str> = d.split("_").collect();
//         let d_qty = d_vec[1].parse().unwrap();

//         let mut d_constraint: Vec<f64> = vec![];
//         for p in 0..problem_size {
//             if s_size >= d_size {
//                 if (p + j)%d_size == 0 { d_constraint.push(1.0); }
//                 else { d_constraint.push(0.0); }
//             }
//             else {
//                 if (p + 2*j)%d_size == 0 { d_constraint.push(1.0); }
//                 else { d_constraint.push(0.0); }
//             }
            
//         }
//         constraints.push(SimplexConstraint::GreaterThan(d_constraint, d_qty));
//     }

//     let elapsed = now1.elapsed();
//     println!("Elapsed: {:.2?}", elapsed);

    

//     // println!("supply_vec: {:?}, size: {}", supply_vec, s_size);
//     // println!("demand_vec: {:?}, size: {}" , demand_vec, d_size);
//     // println!("Costs: {:?}", costs_data);
//     // println!("constraints: {:?}", constraints);

//     let program = Simplex::minimize(&costs_data.costs).with(constraints);

//     let mut simplex = program.unwrap();

//     let now2 = Instant::now();

//     match simplex.solve() {
//         SimplexOutput::UniqueOptimum(x) => println!("{}", x),
//         SimplexOutput::MultipleOptimum(x) => println!("{}", x),
//         _ => panic!("No solution or unbounded"),
//     }

//     let elapsed = now2.elapsed();
//     println!("Elapsed: {:.2?}", elapsed);

//     // for p in 0..problem_size {
//     //     println!("x{}: {}", p, simplex.get_var(p).unwrap());
//     // }
//     println!("target: {}", simplex.get_target().unwrap());
// }

#[allow(dead_code)]
use redis::{Client,Commands};
use std::time::Instant;
use array2d::{Array2D, Error};

mod node;
mod greedy;


fn main () {
    let now1 = Instant::now();

    // raw data retreiving from redis db in docker container and processing them
    //-------------------------------------------------------------------------------------
    let client = Client::open("redis://:alext@127.0.0.1:6379/").unwrap();
    let mut connection = client.get_connection().unwrap();

    let supply_str: String = connection.get("supply").unwrap();
    let demand_str: String = connection.get("demand").unwrap();
    let costs_str: String = connection.get("costs").unwrap();

    let supply_vec: Vec<&str> = supply_str.split(" ").collect();
    let demand_vec: Vec<&str> = demand_str.split(" ").collect();
    let costs_vec: Vec<&str> = costs_str.split(" ").collect();

    let s_task_size = supply_vec.len();
    let d_task_size = demand_vec.len();

    let s_qtys: Vec<_> = supply_vec.into_iter()
                                    .map(|s_node| s_node.split("_")
                                                        .skip(1)
                                                        .collect::<Vec<_>>()[0]
                                                        .parse::<u32>()
                                                        .unwrap())
                                    .collect();
    let d_qtys: Vec<_> = demand_vec.into_iter()
                                    .map(|d_node| d_node.split("_")
                                                        .skip(1)
                                                        .collect::<Vec<_>>()[0]
                                                        .parse::<u32>()
                                                        .unwrap())
                                    .collect();
    // println!("{:?} {:?}", s_qtys, d_qtys);


    // Initialize task vector
    //-------------------------------------------------------------------------------------
    let mut optim_task_vec = Vec::<node::Node>::with_capacity(costs_vec.len());

    //fill task vector with node data
    //-------------------------------------------------------------------------------------
    for node in costs_vec.iter() {
        let cost_data: Vec<_> = node.split("_").collect();
        let s_qt: u16 = s_qtys[cost_data[0].parse::<usize>().unwrap()] as u16;
        let d_qt: u16 = d_qtys[cost_data[1].parse::<usize>().unwrap()] as u16;

        optim_task_vec.push(node::Node::new_with_data(cost_data[0],
                                                cost_data[1],
                                                s_qt,
                                                d_qt,
                                                cost_data[2]));
    }

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
    let mut total_cost = 0.0;

    for row_iter in optim_task_arr.rows_iter() {
        for col in row_iter {
            s_total += col.s_qty;
            d_total += col.d_qty;
            total_asiignment_qty += col.node_qty;
            total_cost += (col.node_qty as f32)*col.node_cost;
        }
    }
    
    println!("Total assignment qty: {}", total_asiignment_qty);
    println!("Total problem cost: {}", total_cost);
    println!("Total left supply: {}", s_total);
    println!("Total left demand: {}", d_total);

    let elapsed = now1.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}