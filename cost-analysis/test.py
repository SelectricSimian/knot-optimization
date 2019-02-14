import knot


def knot_retrieval():
    full_set = knot.KnotSet("trefoil_statistics/all_knots.json")
    good_set = knot.KnotSet("trefoil_statistics/top_100.json")

    best_knot = good_set.knots[10][0]
    equivalent = full_set.retrieve_from_angles(best_knot.angles, best_knot.parity)
    print(equivalent.angles)


def good_adjacency_sizes():
    full_set = knot.KnotSet("trefoil_statistics/all_knots.json")
    good_set = knot.KnotSet("trefoil_statistics/top_100.json")

    good_adjacencies = []

    for good_knot in good_set.one_d_knot_list():
        equivalent = full_set.retrieve_from_angles(good_knot.angles, good_knot.parity)
        adj = full_set.adjacent_knots(equivalent)

        count = 0
        for member in adj:
            if member.cost < 3:
                count += 1

        good_adjacencies.append(count)

    print(sum(good_adjacencies)/len(good_adjacencies))


good_adjacency_sizes()