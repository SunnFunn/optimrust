import random
import numpy as np
import redis
from pulp import *


def check(host, port, db, password):
    r = redis.Redis(host=host, port=port, db=db, password=password)

    supply_str = r.get("supply").decode()
    demand_str = r.get("demand").decode()
    costs_str = r.get("costs").decode()
    initial_solution_str = r.get("initial_solve").decode()

    initial_solution = [int(cost) for cost in initial_solution_str.split("_")[:-1]]

    supply = dict()
    RailRoads_supply = []
    for s_road in supply_str.split(" "):
        idx_road = s_road.split("_")
        supply[idx_road[0]] = int(idx_road[1])
        RailRoads_supply.append((idx_road[0]))
    
    demand = dict()
    RailRoads_demand = []
    for d_road in demand_str.split(" "):
        idx_road = d_road.split("_")
        demand[idx_road[0]] = int(idx_road[1])
        RailRoads_demand.append((idx_road[0]))
    
    roads_costs = []
    costs = []
    for cost in costs_str.split(" "):
        idx1_idx2_cost = cost.split("_")
        roads_costs.append((idx1_idx2_cost[0], idx1_idx2_cost[1], float(idx1_idx2_cost[2])))
        costs.append(float(idx1_idx2_cost[2]))
    
    costs_numpy = np.array(costs)
    costs_numpy_reshaped = costs_numpy.reshape(len(RailRoads_supply), len(RailRoads_demand))
    costs  = costs_numpy_reshaped.tolist()

    #PuLP optim sectio
    #-------------------------------------------------------------------------------------------------
    # The cost data is made into a dictionary
    costs = makeDict([RailRoads_supply, RailRoads_demand], costs, 0)

    # Creates the 'prob' variable to contain the problem data
    prob = LpProblem("RailCars Distribution Problem", LpMinimize)

    # Creates a list of tuples containing all the possible routes for transport
    Routes = [(s, d) for s in RailRoads_supply for d in RailRoads_demand]

    # A dictionary called 'Vars' is created to contain the referenced variables(the routes)
    vars = LpVariable.dicts("Route", (RailRoads_supply, RailRoads_demand), 0, None, LpInteger)
    # print(vars)

    # The objective function is added to 'prob' first
    prob += (
        lpSum([vars[s][d] * costs[s][d] for (s, d) in Routes]),
        "Sum_of_Transporting_Costs",
    )

    # The supply maximum constraints are added to prob for each supply node (warehouse)
    for s in RailRoads_supply:
        prob += (
            lpSum([vars[s][d] for d in RailRoads_demand]) <= supply[s],
            f"Supply_{s}",
        )

    # The demand minimum constraints are added to prob for each demand node (bar)
    for d in RailRoads_demand:
        prob += (
            lpSum([vars[s][d] for s in RailRoads_supply]) >= demand[d],
            f"Demand_{d}",
        )

    # for i in range(len(RailRoads_supply)):
    #     for j in range(len(RailRoads_demand)):
    #         vars[f'{i}'][f'{j}'].setInitialValue(initial_solution[j + i*len(RailRoads_demand)])
    
    # for v in prob.variables():
    #     print(v.varValue)
    
    # The problem is solved using PuLP's choice of Solver
    solver = pulp.PULP_CBC_CMD(msg=True, warmStart=False)
    prob.solve(solver)

    # The status of the solution is printed to the screen
    print("Status:", LpStatus[prob.status])

    # Each of the variables is printed with it's resolved optimum value
    total_assignments = 0
    total_cost = 0
    for v in prob.variables():
    #   print(f"{v}: {v.varValue}")
      if v.varValue:
        total_assignments += v.varValue
    print("Total Assignments = ", total_assignments)

    # The optimised objective function value is printed to the screen
    # print("Total Cost of Transportation = ", value(p.objective))
    print("Total Cost of Transportation = ", value(prob.objective))


if __name__ == "__main__":
    data = check(host="0.0.0.0", port=6379, db=0, password="alext")
    # print(data[:])
