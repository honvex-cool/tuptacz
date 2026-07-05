import "../core/Common.css";
import "./TuptaCh.css";
import { DeckOverlay } from "@deck.gl-community/leaflet";
import { ScatterplotLayer } from "@deck.gl/layers";
import { useMap, MapContainer, TileLayer, useMapEvents } from "react-leaflet";
import { useEffect, useRef, useState } from "react";
import "leaflet/dist/leaflet.css";
import type {
  AlgoEvent,
  AlgorithmSelection,
  ControlEvent,
  Edge,
  FrontendEvent,
  ServerEvent,
  Vertex,
} from "./presentation";
import Slider from "@mui/material/Slider";
import Select from "react-select";

type Phase =
  | "SelectRoutingNetwork"
  | "SelectAlgorithm"
  | "Preprocessing"
  | "SelectQuery"
  | "Query";

const phaseOrder: Record<Phase, number> = {
  SelectRoutingNetwork: 0,
  SelectAlgorithm: 1,
  Preprocessing: 2,
  SelectQuery: 3,
  Query: 4,
};

type GraphProps = {
  vertices: Vertex[];
  edges: Edge[];
  phase: Phase;
  onMapClick: (lat: number, lng: number) => void;
};

function GraphComponent({ vertices, edges, phase, onMapClick }: GraphProps) {
  const map = useMap();

  useMapEvents({
    click(e) {
      if (phase === "SelectQuery" || phase === "Query") {
        onMapClick(e.latlng.lat, e.latlng.lng);
      }
    },
  });

  useEffect(() => {
    const overlay = new DeckOverlay({ views: null, layers: [] });
    map.addLayer(overlay);
    overlay.setProps({
      layers: [
        new ScatterplotLayer({
          id: "nodes",
          data: vertices,
          getPosition: (v: Vertex) => [v.longitude, v.latitude],
          getRadius: 2,
          getFillColor: (_: Vertex) => [255, 0, 0],
        }),
      ],
    });
    return () => {
      map.removeLayer(overlay);
    };
  }, [map, vertices, edges]);

  return null;
}

function Controls({
  phase,
  numStepsInProgress,
  requestStep,
  requestRunToCompletion,
}: {
  phase: Phase;
  numStepsInProgress: number;
  send: (msg: FrontendEvent) => void;
  requestStep: () => void;
  requestRunToCompletion: () => void;
}) {
  const autoplayRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const [autoplaySpeed, setAutoplaySpeed] = useState<number | null>(null);

  function startAutoplay(speed: number) {
    if (autoplayRef.current) clearInterval(autoplayRef.current);
    autoplayRef.current = setInterval(() => {
      requestStep();
    }, 1000 / speed);
    setAutoplaySpeed(speed);
  }

  function stopAutoplay() {
    if (autoplayRef.current) {
      clearInterval(autoplayRef.current);
      setAutoplaySpeed(null);
      autoplayRef.current = null;
    }
  }

  useEffect(() => () => stopAutoplay(), []);

  if (phase === "SelectRoutingNetwork" || phase === "SelectAlgorithm")
    return null;

  const isAutoplay = autoplaySpeed !== null;

  return (
    <div className="controls">
      <button
        className="btn-primary"
        disabled={isAutoplay || numStepsInProgress > 0}
        onClick={() => requestStep()}
      >
        Step
      </button>
      <button
        className="btn-primary"
        onClick={() => {
          if (isAutoplay) {
            stopAutoplay();
          } else {
            startAutoplay(autoplaySpeed ?? 10);
          }
        }}
      >
        {isAutoplay ? "Manual" : "Autoplay"}
      </button>
      <button
        className="btn-primary"
        onClick={() => {
          stopAutoplay();
          requestRunToCompletion();
        }}
      >
        Run to completion
      </button>
      <label>
        Speed:
        <Slider
          min={1}
          max={60}
          value={autoplaySpeed ?? 1}
          onChange={(_, value) => startAutoplay(Number(value as number))}
          disabled={!isAutoplay}
        />
        {autoplaySpeed} fps
      </label>
    </div>
  );
}

function DijkstraSelection({
  setSelection,
}: {
  setSelection: (algorithmSelection: AlgorithmSelection) => void;
}) {
  useEffect(() => {
    setSelection({ type: "Dijkstra", is_bidirectional: false });
  }, []);

  function onChange(isChecked: boolean) {
    setSelection({
      type: "Dijkstra",
      is_bidirectional: isChecked,
    });
  }

  return (
    <label>
      Bidirectional
      <input type="checkbox" onChange={(e) => onChange(e.target.checked)} />
    </label>
  );
}

function AStarSelection({
  setSelection,
}: {
  setSelection: (algorithmSelection: AlgorithmSelection) => void;
}) {
  useEffect(() => {
    setSelection({ type: "AStar", is_bidirectional: false });
  }, []);

  function onChange(isChecked: boolean) {
    setSelection({
      type: "AStar",
      is_bidirectional: isChecked,
    });
  }

  return (
    <label>
      Bidirectional
      <input type="checkbox" onChange={(e) => onChange(e.target.checked)} />
    </label>
  );
}

function ContractionHierarchiesSelection({
  setSelection,
}: {
  setSelection: (algorithmSelection: AlgorithmSelection) => void;
}) {
  useEffect(() => {
    setSelection({ type: "ContractionHierarchies" });
  }, []);

  return <></>;
}

function AlgorithmSelector({
  onSelect,
}: {
  onSelect: (algorithmSelection: AlgorithmSelection) => void;
}) {
  const [selected, setSelected] = useState<string | null>(null);
  const [algorithm, setAlgorithm] = useState<AlgorithmSelection | null>(null);

  const options = ["Dijkstra", "A*", "Contraction Hierarchies"].map((value) => {
    return { value, label: value };
  });
  return (
    <div>
      <h2>Select Algorithm</h2>
      <Select
        placeholder="--choose--"
        options={options}
        onChange={(newValue) => {
          setSelected(newValue?.value!);
        }}
      />
      {selected !== null && (
        <>
          <h3>Configure algorithm</h3>
          {selected === "Dijkstra" && (
            <DijkstraSelection setSelection={setAlgorithm} />
          )}
          {selected === "A*" && <AStarSelection setSelection={setAlgorithm} />}
          {selected === "Contraction Hierarchies" && (
            <ContractionHierarchiesSelection setSelection={setAlgorithm} />
          )}
          <div>
            <button
              disabled={algorithm === null}
              className="btn-primary"
              onClick={(_) => {
                if (algorithm !== null) onSelect(algorithm);
              }}
            >
              Initialize
            </button>
          </div>
        </>
      )}
    </div>
  );
}

function RoutingNetworkSelector({
  availableRoutingNetworkNames,
  onSelect,
}: {
  availableRoutingNetworkNames: string[];
  onSelect: (routingNetworkName: string) => void;
}) {
  return (
    availableRoutingNetworkNames.length > 0 && (
      <div>
        <h2>Select Routing Network</h2>
        <Select
          placeholder="--choose--"
          options={availableRoutingNetworkNames.map((routingNetworkName) => {
            return {
              value: routingNetworkName,
              label: routingNetworkName,
            };
          })}
          onChange={(newValue) => {
            onSelect(newValue?.value!);
          }}
        />
      </div>
    )
  );
}

function TuptaCh() {
  const websocketProtocol =
    window.location.protocol === "https:" ? "wss:" : "ws:";
  const websocketAddress = `${websocketProtocol}//localhost:3001/ws`;
  const ws = useRef<WebSocket | null>(null);

  const [availableRoutingNetworkNames, setAvailableRoutingNetworkNames] =
    useState<string[]>([]);

  const [routingNetworkName, setRoutingNetworkName] = useState<string | null>(
    null,
  );

  const [vertices, setVertices] = useState<Vertex[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);

  const [phase, setPhase] = useState<Phase>("SelectRoutingNetwork");

  const [numStepsInProgress, setNumStepsInProgress] = useState<number>(0);

  function send(event: FrontendEvent) {
    console.log(`sending ${event.type}`);
    ws.current?.send(JSON.stringify(event));
  }

  function requestStep() {
    setNumStepsInProgress((n) => n + 1);
    const type = phase === "Preprocessing" ? "StepPreprocessing" : "StepQuery";
    send({ type });
  }

  function requestRunToCompletion() {
    const type =
      phase === "Preprocessing"
        ? "RunPreprocessingToCompletion"
        : "RunQueryToCompletion";
    send({ type });
  }

  useEffect(() => {
    ws.current = new WebSocket(websocketAddress);
    ws.current.onmessage = (e) => {
      const server_event: ServerEvent = JSON.parse(e.data);
      console.log(`server_event.type: ${server_event.type}`);
      if (server_event.type === "Control") {
        const control_event: ControlEvent = server_event.event;
        console.log(`control_event.type ${control_event.type}`);
        if (control_event.type === "AvailableRoutingNetworks") {
          setAvailableRoutingNetworkNames(control_event.routing_network_names);
        } else if (control_event.type === "RoutingNetworkReady") {
          setPhase("SelectAlgorithm");
        } else if (control_event.type == "PreprocessingReady") {
          setPhase("Preprocessing");
        } else if (control_event.type === "PreprocessingDone") {
          setPhase("SelectQuery");
        } else if (control_event.type === "QueryReady") {
          setPhase("Query");
        } else if (control_event.type === "QueryDone") {
          setPhase("SelectQuery");
        } else if (control_event.type === "ClosestVertexResponse") {
          if (control_event.name == "source") {
            console.log("source settled");
          }
        } else if (control_event.type === "StepDone") {
          setNumStepsInProgress((n) => n - 1);
        }
      } else if (server_event.type === "Algo") {
        const algo_event: AlgoEvent = server_event.event;
        console.log(algo_event);
      }
    };
    return () => ws.current?.close();
  }, []);

  function handleMapClick(latitude: number, longitude: number) {
    if (phase === "SelectQuery") {
      const name = "source";
      const lat_lng = { latitude, longitude };
      send({ type: "ClosestVertexRequest", name, lat_lng });
    }
  }

  function handleRoutingNetworkSelect(newRoutingNetworkName: string) {
    if (newRoutingNetworkName !== routingNetworkName) {
      setRoutingNetworkName(newRoutingNetworkName);
      send({
        type: "SelectRoutingNetwork",
        routing_network_name: newRoutingNetworkName,
      });
    }
  }

  function handleAlgorithmSelect(algorithmSelection: AlgorithmSelection) {
    send({ type: "SelectAlgorithm", algorithm_selection: algorithmSelection });
  }

  return (
    <div className="container">
      <div className="routing-controls">
        {phaseOrder[phase] >= phaseOrder["SelectRoutingNetwork"] && (
          <RoutingNetworkSelector
            availableRoutingNetworkNames={availableRoutingNetworkNames}
            onSelect={handleRoutingNetworkSelect}
          />
        )}

        {phaseOrder[phase] >= phaseOrder["SelectAlgorithm"] && (
          <AlgorithmSelector onSelect={handleAlgorithmSelect} />
        )}

        {phaseOrder[phase] >= phaseOrder["Preprocessing"] && (
          <>
            <Controls
              phase={phase}
              send={send}
              numStepsInProgress={numStepsInProgress}
              requestStep={requestStep}
              requestRunToCompletion={requestRunToCompletion}
            />

            <label>Current phase: {phase}</label>
          </>
        )}
      </div>

      <MapContainer
        center={[50.030641402999564, 19.906958054507893]}
        zoom={13}
        style={{ height: "100vh" }}
        className="map-container"
      >
        <TileLayer
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
        />
        <GraphComponent
          vertices={vertices}
          edges={edges}
          phase={phase}
          onMapClick={handleMapClick}
        />
      </MapContainer>
    </div>
  );
}

export default TuptaCh;
