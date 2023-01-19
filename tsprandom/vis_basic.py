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
        tokens = line.split(" ")
        for token in tokens:
            row.append(int(token))
        dist_matrix.append(row)

net = Network()

for i in range(0, n):
    net.add_node(i, label=str(i), shape='circle')

for i in range(0, n):
    for j in range(0, n):
        if(i==j):
            continue
        net.add_edge(i, j, label=str(dist_matrix[i][j]))
        # value=dist_matrix[i][j],

net.toggle_physics(False)
net.save_graph('cities_vis.html')