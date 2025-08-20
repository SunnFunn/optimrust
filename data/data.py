import random
import redis


def data_initialize(supply_length, demand_length,
                    host, port, db, password):
    r = redis.Redis(host=host, port=port, db=db, password=password)

    # defining list of supply roads and its ids glued in one string
    RailRoads_supply = ""
    total_qty_s = 0
    for i in range(supply_length):
        random_qty_s = random.randint(10,100)
        RailRoads_supply += f"1{i}_{random_qty_s} "

        total_qty_s += random_qty_s

    # defining list of demand roads and its ids glued in one string
    RailRoads_demand = ""
    total_qty_d = 0
    for j in range(demand_length):
        random_qty_d = random.randint(10,100)
        RailRoads_demand += f"2{j}_{random_qty_d} "

        total_qty_d += random_qty_d

    if total_qty_d > total_qty_s:
        d_surplus = total_qty_d - total_qty_s
        RailRoads_supply += f"30_{d_surplus} "
    elif total_qty_d < total_qty_s:
        s_surplus = total_qty_s - total_qty_d
        RailRoads_demand += f"30_{s_surplus} "
    
    RailRoads_supply = RailRoads_supply.strip(" ")
    RailRoads_demand = RailRoads_demand.strip(" ")
    
    costs = ""
    for s in RailRoads_supply.split(" "):
        for d in RailRoads_demand.split(" "):
            random_cost = random.randint(1,16)
            s_id = s.split("_")[0]
            d_id = d.split("_")[0]
            costs += f"{s_id}_{d_id}_{random_cost} "
    
    costs = costs.strip(" ")
    
    r.set("supply", RailRoads_supply.strip(" "))
    r.set("demand", RailRoads_demand.strip(" "))
    r.set("costs", costs)

    
    return RailRoads_supply, RailRoads_demand, costs


if __name__ == "__main__":
    data = data_initialize(supply_length=2, demand_length=2,
                           host="0.0.0.0", port=6379, db=0, password="alext")
    # print(data)
