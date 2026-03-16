import graphviz

def build_diagram():
    g = graphviz.Digraph('flow', graph_attr={'rankdir': 'TB'})

    g.node('A', 'Start')
    g.node('B', 'Process')
    g.node('C', 'Validate')
    g.node('D', 'Store')
    g.node('E', 'End')

    g.edge('A', 'B')
    g.edge('B', 'C')
    g.edge('C', 'D')
    g.edge('D', 'E')
    g.edge('D', 'B', label='retry / resume', style='dashed')

    g.edge('A', 'C', style='invis')
    g.edge('B', 'D', style='invis')

    with g.subgraph(name='cluster_inner') as s:
        s.attr(label='Inner')
        s.node('X', '', shape='plaintext')

    g.edge('E', 'B', constraint='false', label='fix')
    g.edge('E', 'B', constraint='false', label='next task')

    g.render('output', format='png')
    print("Done")

if __name__ == '__main__':
    build_diagram()
