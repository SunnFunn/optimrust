// use std::iter::zip;
use ndarray::*;

#[cfg(test)]
mod test;


#[derive(Clone)]
pub enum SimplexConstraint {
    Equal(Vec<f64>, f64),
    LessThan(Vec<f64>, f64),
    GreaterThan(Vec<f64>, f64),
}

impl SimplexConstraint {
    fn get_vector(&self) -> &Vec<f64> {
        match self {
            SimplexConstraint::Equal(a, _b) => a,
            SimplexConstraint::LessThan(a, _b) => a,
            SimplexConstraint::GreaterThan(a, _b) => a,
        }
    }

    fn get_b(&self) -> f64 {
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
    UniqueOptimum(f64),
    MultipleOptimum(f64),
    InfiniteSolution,
    NoSolution,
}

pub struct SimplexTable {
    pub objective: Vec<f64>,
    pub table: Array2<f64>,
    pub base: Vec<usize>,
    pub vars: Vec<SimplexVar>,
}

impl SimplexTable {
    fn get_entry_var(&self) -> Option<usize> {
        let mut entry_var = None;
        let mut max_entry = -1.0;
        let x_size: usize = self.table.len_of(Axis(0));
        for (i, z) in self.table.row(x_size - 1).iter().enumerate() {
            if i == 0 || i == self.table.ncols() - 1 {
                continue;
            }
            if max_entry < *z {
                max_entry = *z;
                entry_var = Some(i);
            }
        }
        entry_var
    }

    fn get_exit_var(&self, entry_var: usize) -> Option<usize> {
        let mut exit_var = None;
        let mut min_entry = f64::MAX;
        let b = self.table.column(self.table.ncols() - 1);
        for (i, z) in self.table.column(entry_var).iter().enumerate() {
            if i == 0 || i == self.table.nrows() -1 {
                continue;
            }
            if *z <= 0.0 {
                continue;
            }
            if min_entry > b[i] / z {
                min_entry = b[i] / z;
                exit_var = Some(self.base[i - 1]);
            }
        }
        exit_var
    }

    fn step(&mut self, entry_var: usize, exit_var: usize) {
        let exit_row = self.base.iter().position(|x| *x == exit_var).unwrap() + 1;
        let pivot = self.table.row(exit_row)[entry_var];
        {
            let mut row = self.table.row_mut(exit_row);
            row /= pivot;
        }
        for i in 1..self.table.nrows() {
            if i == exit_row {
                continue;
            }
            let mut exit_row = self.table.row(exit_row).to_owned();
            let mut row = self.table.row_mut(i);
            let factor = row[entry_var] / -1.0;
            exit_row *= factor;
            row += &exit_row;
        }
        self.base = self
            .base
            .iter_mut()
            .map(|x| if *x == exit_var { entry_var } else { *x })
            .collect();
    }

    pub fn solve(&mut self) -> SimplexOutput {
        loop {
            if let Some(entry_var) = self.get_entry_var() {
                if let Some(exit_var) = self.get_exit_var(entry_var) {
                    self.step(entry_var, exit_var);
                } else {
                    return SimplexOutput::InfiniteSolution;
                }
            } else {
                panic!("Can't continue");
            }
            let mut optimum = true;
            let mut unique = true;
            let nrows = self.table.len_of(Axis(0));
            for (i, &z) in self.table.row(nrows - 1).iter().skip(1).enumerate() {
                optimum = optimum && z <= 0.0;
                if !self.base.contains(&i) && i < self.objective.len() {
                    unique = unique && z - self.objective[i] < 0.0;
                }
            }
            if optimum {
                let optimum = self.table.row(0)[self.table.ncols() - 1];
                for (i, var) in self.base.iter().enumerate() {
                    if self.vars[*var - 1].is_artificial() {
                        if self.table.row(i + 1)[self.table.ncols() - 1] > 0.0 {
                            /* Artificial variable might have taken slack var value */
                            if self.vars[*var - 2].is_slack() {
                                if self.table.row(0)[*var - 1] == 0.0 {
                                    continue;
                                }
                            }
                            return SimplexOutput::NoSolution;
                        }
                    }
                }
                // return SimplexOutput::UniqueOptimum(optimum);
                if unique {
                    return SimplexOutput::UniqueOptimum(optimum);
                } else {
                    return SimplexOutput::MultipleOptimum(optimum);
                }
            }
        }
    }

    pub fn get_var(&self, var: usize) -> Option<f64> {
        if var > self.objective.len() {
            return None;
        }
        for (i, v) in self.base.iter().enumerate() {
            if *v == var {
                return Some(self.table.row(i + 1)[self.table.ncols() - 1]);
            }
        }
        return Some(0.0);
    }
}

pub struct SimplexMinimizerBuilder {
    objective: Vec<f64>,
}

impl SimplexMinimizerBuilder {
    pub fn with(self, constraints: Vec<SimplexConstraint>) -> Result<SimplexTable, String> {
        let mut table = Vec::new();
        let mut vars = Vec::new();
        let m_big: f64 = 1000000.0;
        table.push(1.0);
        for var in self.objective.iter() {
            // table.push(var * -1.0);
            table.push(*var);
            vars.push(SimplexVar::Real);
        }
        for (i, constraint) in constraints.iter().enumerate() {
            match constraint {
                SimplexConstraint::LessThan(_, _) => {
                    table.push(0.0);
                    vars.push(SimplexVar::Slack(i));
                }
                SimplexConstraint::GreaterThan(_, _) => {
                    table.push(0.0);
                    vars.push(SimplexVar::NegativeSlack(i));
                    // table.push(f64::MIN);
                    table.push(m_big.clone());
                    vars.push(SimplexVar::Artificial(i));
                }
                _ => {
                    // table.push(f64::MIN);
                    table.push(m_big.clone());
                    vars.push(SimplexVar::Artificial(i));
                }
            }
            // table.push(f64::MIN);
            // vars.push(SimplexVar::Artificial(i));
        }
        table.push(0.0);

        for (i, constraint) in constraints.iter().enumerate() {
            table.push(0.0);
            for a in constraint.get_vector() {
                table.push(*a);
            }
            for var in vars.iter() {
                match var {
                    SimplexVar::Slack(j) => {
                        if *j == i {
                            table.push(1.0);
                        } else {
                            table.push(0.0);
                        }
                    }
                    SimplexVar::NegativeSlack(j) => {
                        if *j == i {
                            table.push(-1.0);
                        } else {
                            table.push(0.0);
                        }
                    }
                    SimplexVar::Artificial(j) => {
                        if *j == i {
                            table.push(1.0);
                        } else {
                            table.push(0.0);
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
            .filter_map(|(i, x)| if x.is_artificial() || x.is_slack() { Some(i + 1) } else { None })
            .collect();

        let mut target: f64 = 0.0;
        // for it in zip(constraints.clone, base.clone()) {
        //     target += it.0.get_b()*table[it.1];
        // }
        for (i, constraint) in constraints.iter().enumerate() {
            target += constraint.get_b()*table[base[i]];
        }
        println!{"{}", target};
        table.push(target);

        for i in 0..vars.len() {
            let mut delta: f64 = 0.0;
            for (j, b) in base.iter().enumerate() {
                delta += table[(j + 1)*(vars.len() + 2) + i + 1]*table[*b];
            }
            // println!{"delta: {}, Cj: {}", delta, table[i+1]};
            delta = delta - table[i+1];
            table.push(delta);
        }
        table.push(0.0);

        let table = Array2::from_shape_vec((base.len() + 2, vars.len() + 2), table);

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
    pub fn minimize(objective: &Vec<f64>) -> SimplexMinimizerBuilder {
        SimplexMinimizerBuilder {
            objective: objective.clone(),
        }
    }
}

fn main(){
    let costs = vec![3.0, 4.0, 3.0];
    let program = Simplex::minimize(&costs)
    .with(vec![
        SimplexConstraint::LessThan(vec![2.0, 1.0, 1.0], 2.0),
        SimplexConstraint::GreaterThan(vec![3.0, 8.0, 2.0], 1.0),
        SimplexConstraint::GreaterThan(vec![0.0, 1.0, 1.0], 2.0),
    ]);

    let mut simplex = program.unwrap();
    // let entr: usize = simplex.get_entry_var().unwrap();
    // let ext: usize = simplex.get_exit_var(entr).unwrap();

    // println!("{:?}", simplex.objective);
    // println!("{:?}", simplex.vars);
    // println!("{:?}", simplex.base);
    // println!("{:?}", simplex.table);
    // println!("entry: {:?}, exit: {:?}", entr, ext);

    // println!("nrows: {:?}", simplex.table.len_of(Axis(0)));

    // simplex.step(entr, ext);

    // let entr: usize = simplex.get_entry_var().unwrap();
    // let ext: usize = simplex.get_exit_var(entr).unwrap();

    // println!("{:?}", simplex.objective);
    // println!("{:?}", simplex.vars);
    // println!("{:?}", simplex.base);
    // println!("{:?}", simplex.table);
    // println!("entry: {:?}, exit: {:?}", entr, ext);

    // simplex.step(entr, ext);

    // println!("{:?}", simplex.objective);
    // println!("{:?}", simplex.vars);
    // println!("{:?}", simplex.base);
    // println!("{:?}", simplex.table);

    match simplex.solve() {
        SimplexOutput::UniqueOptimum(x) => println!("{}", x),
        SimplexOutput::MultipleOptimum(x) => println!("{}", x),
        _ => panic!("No solution or unbounded"),
    }
    println!("{:?}", simplex.get_var(1));
    println!("{:?}", simplex.get_var(2));
    println!("{:?}", simplex.get_var(3));
    // println!("{:?}", simplex.get_var(4));
}