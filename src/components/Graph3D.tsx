import { useEffect, useState, useMemo } from "react";
import { Canvas } from "@react-three/fiber";
import { OrbitControls, Html, Line } from "@react-three/drei";
import {
  forceSimulation,
  forceLink,
  forceManyBody,
  forceCenter,
} from "d3-force-3d";
import { GraphData, FileNode } from "../lib/graph";

// D3 injects physics coordinates directly into node objects
export interface D3Node extends FileNode {
  x: number;
  y: number;
  z: number;
  vx?: number;
  vy?: number;
  vz?: number;
}

// D3 mutates the links array and replaces the string IDs with actual node object references.
export interface D3Link {
  source: string | D3Node;
  target: string | D3Node;
}

function getNodeColor(role: string): string {
  switch (role) {
    case "Application Entry":
      return "#10b981";
    case "Core Orchestrator":
      return "#f43f5e";
    case "Primitive / Foundation":
      return "#8b5cf6";
    case "Isolated / Dead Code":
      return "#475569";
    default:
      return "#3b82f6";
  }
}

function getNodeSize(node: FileNode): number {
  const networkTraffic = node.in_degree + node.out_degree;
  return 2 + Math.min(networkTraffic * 0.4, 15);
}

export default function Graph3D({ data }: { data: GraphData }) {
  const [isEngineReady, setIsEngineReady] = useState(false);
  const [hoveredNode, setHoveredNode] = useState<D3Node | null>(null);

  const simulationData = useMemo(() => {
    if (!data.nodes.length) return null;

    const d3Nodes = data.nodes as D3Node[];
    const d3Links = data.links as D3Link[];

    const sim = forceSimulation(d3Nodes, 3)
      .force(
        "link",
        forceLink(d3Links)
          .id((d: D3Node) => d.id)
          .distance(60),
      )
      .force("charge", forceManyBody().strength(-200))
      .force("center", forceCenter(0, 0, 0));

    sim.tick(150);
    sim.stop();

    return { nodes: d3Nodes, links: d3Links };
  }, [data]);

  useEffect(() => {
    if (simulationData) setIsEngineReady(true);
  }, [simulationData]);

  const neighborMap = useMemo(() => {
    if (!simulationData) return new Set<string>();
    const neighbors = new Set<string>();

    if (hoveredNode) {
      neighbors.add(hoveredNode.id);
      simulationData.links.forEach((link: D3Link) => {
        if (typeof link.source === "string" || typeof link.target === "string")
          return;
        if (link.source.id === hoveredNode.id) neighbors.add(link.target.id);
        if (link.target.id === hoveredNode.id) neighbors.add(link.source.id);
      });
    }
    return neighbors;
  }, [hoveredNode, simulationData]);

  if (!isEngineReady || !simulationData) {
    return (
      <div className="text-slate-500 font-mono text-sm h-full flex items-center justify-center animate-pulse">
        Calculating Physics...
      </div>
    );
  }

  return (
    <Canvas camera={{ position: [0, 0, 300], fov: 60 }}>
      <color attach="background" args={["#020617"]} />
      <ambientLight intensity={0.4} />
      <pointLight position={[200, 200, 200]} intensity={2} />

      <OrbitControls makeDefault enableDamping dampingFactor={0.1} />

      {hoveredNode &&
        simulationData.links.map((link: D3Link, i: number) => {
          if (
            typeof link.source === "string" ||
            typeof link.target === "string"
          )
            return null;

          if (
            link.source.id !== hoveredNode.id &&
            link.target.id !== hoveredNode.id
          )
            return null;
          if (link.source.x === undefined || link.target.x === undefined)
            return null;

          const isOutgoing = link.source.id === hoveredNode.id;

          return (
            <Line
              key={`link-${i}`}
              points={[
                [link.source.x, link.source.y, link.source.z],
                [link.target.x, link.target.y, link.target.z],
              ]}
              color={isOutgoing ? "#10b981" : "#f59e0b"}
              lineWidth={2}
              transparent
              opacity={0.8}
            />
          );
        })}

      {simulationData.nodes.map((node: D3Node) => {
        const isHovered = hoveredNode?.id === node.id;
        const isNeighbor = neighborMap.has(node.id);

        let opacity = 1;
        if (hoveredNode) {
          opacity = isNeighbor ? 1 : 0.05;
        } else if (node.centrality_role === "Standard Node") {
          opacity = 0.4;
        }

        return (
          <mesh
            key={node.id}
            position={[node.x, node.y, node.z]}
            onPointerOver={(e) => {
              e.stopPropagation();
              setHoveredNode(node);
              document.body.style.cursor = "pointer";
            }}
            onPointerOut={(e) => {
              e.stopPropagation();
              setHoveredNode(null);
              document.body.style.cursor = "default";
            }}
          >
            <sphereGeometry args={[getNodeSize(node), 16, 16]} />
            <meshStandardMaterial
              color={isHovered ? "#ffffff" : getNodeColor(node.centrality_role)}
              transparent
              opacity={opacity}
              emissive={
                isNeighbor && hoveredNode
                  ? getNodeColor(node.centrality_role)
                  : "#000000"
              }
              emissiveIntensity={0.5}
            />

            {isHovered && (
              <Html
                distanceFactor={100}
                zIndexRange={[100, 0]}
                className="pointer-events-none"
              >
                <div className="bg-slate-900 border border-slate-700 text-slate-200 px-4 py-3 rounded shadow-2xl -translate-x-1/2 -translate-y-full mb-4 w-max">
                  <div className="font-bold text-indigo-400 mb-1">
                    {node.name}
                  </div>
                  <div className="text-xs text-slate-400 mb-2">
                    {node.centrality_role}
                  </div>
                  <div className="flex gap-4 text-xs font-mono border-t border-slate-800 pt-2">
                    <div className="flex flex-col">
                      <span className="text-slate-500">Imports</span>
                      <span className="text-slate-300">{node.out_degree}</span>
                    </div>
                    <div className="flex flex-col">
                      <span className="text-slate-500">Imported By</span>
                      <span className="text-slate-300">{node.in_degree}</span>
                    </div>
                  </div>
                </div>
              </Html>
            )}
          </mesh>
        );
      })}
    </Canvas>
  );
}
