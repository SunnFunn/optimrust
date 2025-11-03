import random
import redis


def generater_random_list(rand_list, total_amount, node_amount, length):
    for i in range(length):
        rand_num = random.randint(1, node_amount)
        rand_list.append(rand_num)
        if i == length - 1 and sum(rand_list) < total_amount:
            diff = total_amount - sum(rand_list)
            residue = diff%length
            share = diff//length
            rand_list = [ele + share for ele in rand_list]
            rand_list[-1] += residue
    return rand_list


def data_initialize(supply_length, demand_length,
                    host, port, db, password):
    r = redis.Redis(host=host, port=port, db=db, password=password)

    s_node_qty = 30

    # defining list of supply roads and its ids glued in one string
    RailRoads_supply = ""
    supply_list =  generater_random_list(rand_list=[],
                                total_amount=supply_length*s_node_qty,
                                node_amount=s_node_qty,
                                length=supply_length)
    for i in range(supply_length):
        RailRoads_supply += f"{i}_{supply_list[i]} "

    # defining list of demand roads and its ids glued in one string
    RailRoads_demand = ""
    demand_list =  generater_random_list(rand_list=[],
                                total_amount=demand_length*s_node_qty,
                                node_amount=s_node_qty,
                                length=demand_length)
    for i in range(demand_length):
        RailRoads_demand += f"{i}_{demand_list[i]} "
    
    RailRoads_supply = RailRoads_supply.strip(" ")
    RailRoads_demand = RailRoads_demand.strip(" ")
    
    costs = ""
    for s in RailRoads_supply.split(" "):
        for d in RailRoads_demand.split(" "):
            random_cost = random.randint(5,30)
            s_id = s.split("_")[0]
            d_id = d.split("_")[0]
            if s_id == d_id:
                random_cost = random.randint(1,3)
            costs += f"{s_id}_{d_id}_{random_cost} "
    
    costs = costs.strip(" ")
    
    r.set("supply", RailRoads_supply.strip(" "))
    r.set("demand", RailRoads_demand.strip(" "))
    r.set("costs", costs)

    
    return RailRoads_supply, RailRoads_demand, costs


if __name__ == "__main__":
    # node_amount = 30
    # length = 2
    # rand_list =  generater_random_list(rand_list=[],
    #                             total_amount=length*node_amount,
    #                             node_amount=node_amount,
    #                             length=length)
    data = data_initialize(supply_length=1000, demand_length=1000,
                           host="0.0.0.0", port=6379, db=0, password="alext")
    # print(data[:])
