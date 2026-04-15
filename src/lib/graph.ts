export interface FileNode {
  id: string;
  name: string;
  imports: string[];
  in_degree: number;
  out_degree: number;
  centrality_role: string;
  is_entry_point: boolean;
}

export interface GraphLink {
  source: string;
  target: string;
}

export interface GraphData {
  nodes: FileNode[];
  links: GraphLink[];
}

export function generateGraphData(nodes: FileNode[]): GraphData {
  const links: GraphLink[] = [];

  // Deep clone the nodes so D3 can safely mutate them with x,y,z coordinates
  const d3Nodes = nodes.map((n) => ({ ...n }));

  d3Nodes.forEach((sourceNode) => {
    d3Nodes.forEach((targetNode) => {
      if (sourceNode.id === targetNode.id) return;

      const targetStem = targetNode.name.split(".")[0];
      const hasDependency = sourceNode.imports.some((imp) =>
        imp.includes(targetStem),
      );

      if (hasDependency) {
        links.push({
          source: sourceNode.id,
          target: targetNode.id,
        });
      }
    });
  });

  return { nodes: d3Nodes, links };
}
