use redis::{Client,Commands};
use array2d::{Array2D, Error};

use crate::node;

pub fn get() -> Result<(Vec::<node::Node>, usize, usize), String> {
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

    let s_qtys: Vec<_> = supply_vec.iter()
                                    .map(|s_node| s_node.split("_")
                                                        .skip(1)
                                                        .collect::<Vec<_>>()[0]
                                                        .parse::<i32>()
                                                        .unwrap())
                                    .collect();
    let d_qtys: Vec<_> = demand_vec.iter()
                                    .map(|d_node| d_node.split("_")
                                                        .skip(1)
                                                        .collect::<Vec<_>>()[0]
                                                        .parse::<i32>()
                                                        .unwrap())
                                    .collect();


    // Initialize task vector
    //-------------------------------------------------------------------------------------
    let mut optim_task_vec = Vec::<node::Node>::with_capacity(costs_vec.len());

    //fill task vector with node data
    //-------------------------------------------------------------------------------------
    for node in costs_vec.iter() {
        let cost_data: Vec<_> = node.split("_").collect();
        let s_qt: i32 = s_qtys[cost_data[0].parse::<usize>().unwrap()] as i32;
        let d_qt: i32 = d_qtys[cost_data[1].parse::<usize>().unwrap()] as i32;

        optim_task_vec.push(node::Node::new_with_data(cost_data[0],
                                                cost_data[1],
                                                s_qt,
                                                d_qt,
                                                cost_data[2]));
    }
    Ok((optim_task_vec, s_task_size, d_task_size))
}