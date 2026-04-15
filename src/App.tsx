import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { generateGraphData, FileNode, GraphData } from "./lib/graph";
import Graph3D from "./components/Graph3D";

export default function App() {
  const [targetDir, setTargetDir] = useState("");
  const [status, setStatus] = useState("Ready for directory.");
  const [isScanning, setIsScanning] = useState(false);
  const [nodes, setNodes] = useState<FileNode[]>([]);
  const [graphData, setGraphData] = useState<GraphData | null>(null);

  async function handleScan() {
    if (!targetDir) return;

    setIsScanning(true);
    setStatus("Mapping codebase architecture...");
    setNodes([]);

    try {
      const result = await invoke<{ nodes: FileNode[] }>("map_codebase", {
        dirPath: targetDir,
      });

      setNodes(result.nodes);

      const formattedData = generateGraphData(result.nodes);
      setGraphData(formattedData);

      setStatus(`Successfully mapped ${result.nodes.length} nodes.`);
    } catch (error) {
      setStatus(`Error: ${error}`);
    } finally {
      setIsScanning(false);
    }
  }
  return (
    <div className="flex h-screen w-screen bg-slate-950 text-slate-300 font-sans overflow-hidden">
      <div className="w-80 bg-slate-900 border-r border-slate-800 flex flex-col p-4 z-10 shadow-xl">
        <h1 className="text-xl font-bold text-white mb-6 tracking-wide">
          Codebase Cartographer
        </h1>

        <div className="flex flex-col gap-3 mb-6">
          <label className="text-xs uppercase tracking-wider text-slate-500 font-semibold">
            Target Directory
          </label>
          <input
            type="text"
            value={targetDir}
            onChange={(e) => setTargetDir(e.target.value)}
            placeholder="/absolute/path/to/repo"
            className="w-full p-2 bg-slate-950 border border-slate-700 rounded text-sm focus:outline-none focus:border-indigo-500 transition-colors"
          />
          <button
            onClick={handleScan}
            disabled={isScanning}
            className="w-full py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded text-sm font-semibold disabled:opacity-50 transition-colors"
          >
            {isScanning ? "Scanning..." : "Generate Map"}
          </button>
        </div>

        <div className="text-xs text-indigo-400 font-mono mb-2">{status}</div>

        <div className="flex-1 overflow-y-auto border border-slate-800 rounded bg-slate-950/50 p-2">
          {nodes.length > 0 ? (
            <ul className="text-xs font-mono space-y-1">
              {nodes.map((node, index) => {
                return (
                  <li
                    key={index}
                    className="truncate hover:text-indigo-400 cursor-pointer transition-colors p-1 rounded hover:bg-slate-900"
                    title={node.id}
                    onClick={() => {
                      setStatus(`Viewing ${node.name} dependencies...`);
                      console.log(`\n--- Dependencies for ${node.name} ---`);
                      console.log(`Path: ${node.id}`);
                      console.log(`Imports Found: ${node.imports.length}`);
                      console.dir(node.imports);
                    }}
                  >
                    {node.name}
                  </li>
                );
              })}
            </ul>
          ) : (
            <div className="h-full flex items-center justify-center text-slate-600 text-sm italic">
              No files mapped yet.
            </div>
          )}
        </div>
      </div>
      <div className="flex-1 relative bg-slate-950 flex items-center justify-center">
        {graphData && graphData.nodes.length > 0 ? (
          <Graph3D data={graphData} />
        ) : (
          <div className="text-center z-10 border border-slate-800 bg-slate-900/80 p-8 rounded-lg shadow-2xl backdrop-blur-sm">
            <h2 className="text-2xl font-bold text-slate-400 mb-2">
              Awaiting Target Directory...
            </h2>
          </div>
        )}
      </div>
    </div>
  );
}
