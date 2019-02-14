import knot


def main():
    knot_set = knot.KnotSet("trefoil_statistics/top_100.json")
    print(knot_set.knots[10][0].cost)
    adj_set = knot_set.adjacent_knots(knot_set.knots[10][0])
    for adj_knot in adj_set:
        print(adj_knot.angles)


main()
