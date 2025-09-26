import random
import redis


def data_initialize(supply_length, demand_length,
                    host, port, db, password):
    r = redis.Redis(host=host, port=port, db=db, password=password)

    Total_Qty = 0

    # defining list of supply roads and its ids glued in one string
    RailRoads_supply = ""
    total_qty_s = 0
    for i in range(supply_length):
        random_qty_s = random.randint(1,50)
        RailRoads_supply += f"{i}_{random_qty_s} "

        total_qty_s += random_qty_s

    # defining list of demand roads and its ids glued in one string
    RailRoads_demand = ""
    total_qty_d = 0
    for j in range(demand_length):
        random_qty_d = random.randint(1,10)
        RailRoads_demand += f"{j}_{random_qty_d} "

        total_qty_d += random_qty_d

    if total_qty_d > total_qty_s:
        d_surplus = total_qty_d - total_qty_s
        RailRoads_supply += f"{supply_length}_{d_surplus} "

        Total_Qty += total_qty_s
        Total_Qty += total_qty_d
        Total_Qty += d_surplus
        print(Total_Qty)

    elif total_qty_d < total_qty_s:
        s_surplus = total_qty_s - total_qty_d
        RailRoads_demand += f"{demand_length}_{s_surplus} "

        Total_Qty += total_qty_s
        Total_Qty += total_qty_d
        Total_Qty += s_surplus
        print(Total_Qty)
    
    RailRoads_supply = RailRoads_supply.strip(" ")
    RailRoads_demand = RailRoads_demand.strip(" ")
    
    costs = ""
    for s in RailRoads_supply.split(" "):
        for d in RailRoads_demand.split(" "):
            random_cost = random.randint(1,10)
            s_id = s.split("_")[0]
            d_id = d.split("_")[0]
            costs += f"{s_id}_{d_id}_{random_cost} "
    
    costs = costs.strip(" ")
    
    r.set("supply", RailRoads_supply.strip(" "))
    r.set("demand", RailRoads_demand.strip(" "))
    r.set("costs", costs)

    
    return RailRoads_supply, RailRoads_demand, costs


if __name__ == "__main__":
    data = data_initialize(supply_length=150, demand_length=150,
                           host="0.0.0.0", port=6379, db=0, password="alext")
    # print(data[:])
