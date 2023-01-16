from colorsys import hsv_to_rgb
from pyvis.network import Network
from operator import itemgetter
import sys

def cmp_permutations(perm1, perm2):
    # invert first permutation
    perm_1_inv = perm1[:]
    for i in range(0, len(perm1)):
        perm_1_inv[perm1[i]] = i

    # Compose the two permutations
    p = [0 for i in range(0, len(perm1))]
    for i in range(0, len(perm1)):
        p[i] = perm2[perm_1_inv[i]]

    count = 0;
    for i in range(0, len(perm1)):
        while p[i] != i :
            a = p[p[i]]
            b = p[i]
            p[a], p[b] = p[b], p[a]
            count += 1

    return count

nodes = []

N = 40

if len(sys.argv) < 2:
    print("Pass input file name as parameter")
    raise Exception

with open(sys.argv[1]) as f:
    lines = f.readlines()
    for i, line in enumerate(lines[1:]):
        tokens = line.split(";")
        id = int(tokens[0])
        tour = eval(tokens[1])
        val = int(tokens[2])
        sp = int(tokens[3])

        nodes.append((id, tour, val, sp))

nodes.sort(key=itemgetter(2))

net = Network()

best_nodes = nodes[0:N]

min_val = min(map(itemgetter(2), best_nodes))
max_val = max(map(itemgetter(2), best_nodes))

min_sp = min(map(itemgetter(3), best_nodes))
max_sp = max(map(itemgetter(3), best_nodes))

base_size = 25

for i, node in enumerate(best_nodes):
    vrange = max_val - min_val
    svalue = (node[2] - min_val) / vrange
    rgb = hsv_to_rgb((1.0 - svalue) * 0.3, 1.0, 1.0)
    rgb = (int(rgb[0] * 255), int(rgb[1] * 255), int(rgb[2] * 255))

    if node[2] == min_val:
        label = '*' + str(node[2])
    else:
        label = str(node[2])

    net.add_node(
        i,
        label=label,
        title=f"Tour: {node[1]}, Tour len: {node[2]}, {node[3]} starting points led to this LO",
        size=(node[3] / max_sp) * base_size,
        color="#%02x%02x%02x" % rgb,
    )

for i, node in enumerate(best_nodes[1:]):
    swap_cnt = cmp_permutations(best_nodes[0][1], node[1])
    net.add_edge(0, i+1, label=str(swap_cnt), color="#888888")


net.toggle_physics(True)

name = "local_optima_graph"

if len(sys.argv) > 2:
    name = sys.argv[2]

net.write_html(f'{name}.html', local=True, notebook=False)