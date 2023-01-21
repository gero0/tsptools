import sys
from pyvis.network import Network

if len(sys.argv) < 2:
    print("Pass input file name as parameter")
    raise Exception

dist_matrix = []

with open(sys.argv[1]) as f:
    lines = f.readlines()
    n = int(lines[0])
    for i, line in enumerate(lines[1:n+1]):
        row = []
        tokens = line.split("\t")
        for token in tokens:
            row.append(int(token))
        dist_matrix.append(row)

net = Network()

net.add_node(0, label="0", shape="circle")

for i in range(1, n):
    net.add_node(i, label=str(i), shape='circle', mass=dist_matrix[0][i] / 2)

net.toggle_physics(False)
net.save_graph('cities_vis.html')