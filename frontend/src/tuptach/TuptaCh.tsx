import "../core/Common.css";
import "./TuptaCh.css";
import { DeckOverlay } from "@deck.gl-community/leaflet";
import { PathLayer, ScatterplotLayer } from "@deck.gl/layers";
import { useMap, MapContainer, TileLayer, useMapEvents } from "react-leaflet";
import { useEffect, useRef, useState } from "react";
import "leaflet/dist/leaflet.css";
import type {
  AlgoEvent,
  AlgorithmSelection,
  ControlEvent,
  Edge,
  FrontendEvent,
  HighlightMode,
  LatLng,
  ServerEvent,
} from "./presentation";
import Slider from "@mui/material/Slider";
import Select from "react-select";

type Phase =
  | "SelectRoutingNetwork"
  | "RoutingNetworkSelected"
  | "SelectAlgorithm"
  | "AlgorithmSelected"
  | "Preprocessing"
  | "SelectQuery"
  | "QuerySelected"
  | "Query"
  | "QueryDone";

const phaseOrder: Record<Phase, number> = {
  SelectRoutingNetwork: 0,
  RoutingNetworkSelected: 1,
  SelectAlgorithm: 2,
  AlgorithmSelected: 3,
  Preprocessing: 4,
  SelectQuery: 5,
  QuerySelected: 6,
  Query: 7,
  QueryDone: 8,
};

type MapPoint = {
  id: number;
  longitude: number;
  latitude: number;
  color: [number, number, number];
  radius: number;
};

type MapEdge = {
  id: number;
  path: [number, number][];
  color: [number, number, number];
  width: number;
};

function highlightColor(mode: HighlightMode): [number, number, number] {
  switch (mode) {
    case "Visited":
      return [255, 80, 80];
    case "Awaiting":
      return [0, 120, 255];
    case "Source":
      return [0, 255, 0];
  }
}

type GraphProps = {
  mapPoints: Map<number, MapPoint>;
  mapEdges: Map<number, MapEdge>;
  path: Edge[] | null;
  phase: Phase;
  onMapClick: (lat: number, lng: number) => void;
  source: QueryPoint | null;
  target: QueryPoint | null;
  pendingPoint: QueryPoint | null;
};

function GraphComponent({
  mapPoints,
  mapEdges,
  path,
  phase,
  onMapClick,
  source,
  target,
  pendingPoint,
}: GraphProps) {
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
        new PathLayer({
          id: "edges",
          data: Array.from(mapEdges.values()),
          getPath: (e: MapEdge) => e.path,
          getColor: (e: MapEdge) => e.color,
          getWidth: (e: MapEdge) => e.width,
          widthMinPixels: 1,
        }),
        new PathLayer({
          id: "path",
          data: path ?? [],
          getPath: (e: Edge) =>
            e.props.points.map(
              (p) => [p.longitude, p.latitude] as [number, number],
            ),
          getColor: [255, 0, 0],
          getWidth: 4,
          widthMinPixels: 3,
        }),
        new ScatterplotLayer({
          id: "nodes",
          data: Array.from(mapPoints.values()),
          getPosition: (p: MapPoint) => [p.longitude, p.latitude],
          getRadius: (p: MapPoint) => p.radius,
          getFillColor: (p: MapPoint) => p.color,
          radiusMinPixels: 4,
        }),
        new ScatterplotLayer({
          id: "source",
          data: source ? [source] : [],
          getPosition: (p: QueryPoint) => [
            p.lat_lng.longitude,
            p.lat_lng.latitude,
          ],
          getRadius: 8,
          getFillColor: [0, 255, 0],
          radiusMinPixels: 4,
        }),
        new ScatterplotLayer({
          id: "target",
          data: target ? [target] : [],
          getPosition: (p: QueryPoint) => [
            p.lat_lng.longitude,
            p.lat_lng.latitude,
          ],
          getRadius: 8,
          getFillColor: [0, 0, 255],
          radiusMinPixels: 4,
        }),
        new ScatterplotLayer({
          id: "pending",
          data: pendingPoint ? [pendingPoint] : [],
          getPosition: (p: QueryPoint) => [
            p.lat_lng.longitude,
            p.lat_lng.latitude,
          ],
          getRadius: 8,
          getFillColor: [255, 165, 0],
          radiusMinPixels: 4,
        }),
      ],
    });
    return () => {
      map.removeLayer(overlay);
    };
  }, [map, mapPoints, mapEdges, path, source, target, pendingPoint]);

  return null;
}

function Controls({
  numStepsInProgress,
  requestStep,
  requestRunToCompletion,
}: {
  numStepsInProgress: number;
  requestStep: () => void;
  requestRunToCompletion: () => void;
}) {
  const autoplayRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const [autoplaySpeed, setAutoplaySpeed] = useState<number | null>(null);
  const [isRunToCompletionRequested, setIsRunToCompletionRequested] =
    useState<boolean>(false);

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
      <div>
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
      </div>
      <div>
        <button
          disabled={isRunToCompletionRequested}
          className="btn-primary"
          onClick={() => {
            setIsRunToCompletionRequested(true);
            stopAutoplay();
            requestRunToCompletion();
          }}
        >
          Run to completion
        </button>
      </div>
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

type QueryPoint = {
  id: number;
  lat_lng: LatLng;
};

type QuerySelectorProps = {
  onSubmit: (sourceId: number, targetId: number) => void;
  source: QueryPoint | null;
  setSource: (s: QueryPoint | null) => void;
  target: QueryPoint | null;
  setTarget: (t: QueryPoint | null) => void;
  pendingPoint: QueryPoint | null;
  setPendingPoint: (p: QueryPoint | null) => void;
  selecting: "source" | "target" | null;
  setSelecting: (s: "source" | "target" | null) => void;
};

function QuerySelector({
  onSubmit,
  source,
  setSource,
  target,
  setTarget,
  pendingPoint,
  setPendingPoint,
  selecting,
  setSelecting,
}: QuerySelectorProps) {
  function reset() {
    setSource(null);
    setTarget(null);
    setPendingPoint(null);
    setSelecting(null);
  }

  function startSelectingSource() {
    setSelecting("source");
  }

  function startSelectingTarget() {
    setSelecting("target");
  }

  function acceptPending() {
    if (pendingPoint === null) return;
    if (selecting === "source") setSource(pendingPoint);
    else if (selecting === "target") setTarget(pendingPoint);
    setPendingPoint(null);
    setSelecting(null);
  }

  return (
    <div>
      <h5>Select Query Points</h5>
      <div>
        <button
          className="btn-primary"
          onClick={startSelectingSource}
          disabled={selecting === "source"}
        >
          {selecting === "source" ? "Click map for source..." : "Pick Source"}
        </button>
        {source !== null && <span> ✓ Source set</span>}
      </div>

      {selecting === "source" && pendingPoint !== null && (
        <div>
          <span>
            Nearest: ({pendingPoint.lat_lng.latitude.toFixed(4)},{" "}
            {pendingPoint.lat_lng.longitude.toFixed(4)})
          </span>
          <button className="btn-primary" onClick={acceptPending}>
            Accept
          </button>
        </div>
      )}

      <div>
        <button
          className="btn-primary"
          onClick={startSelectingTarget}
          disabled={selecting === "target" || source === null}
        >
          {selecting === "target" ? "Click map for target..." : "Pick Target"}
        </button>
        {target !== null && <span> ✓ Target set</span>}
      </div>

      {selecting === "target" && pendingPoint !== null && (
        <div>
          <span>
            Nearest: ({pendingPoint.lat_lng.latitude.toFixed(4)},{" "}
            {pendingPoint.lat_lng.longitude.toFixed(4)})
          </span>
          <button className="btn-primary" onClick={acceptPending}>
            Accept
          </button>
        </div>
      )}

      <div>
        <button className="btn-primary" onClick={reset}>
          Reset
        </button>
        <button
          className="btn-primary"
          disabled={source === null || target === null}
          onClick={() => {
            if (source !== null && target !== null) {
              onSubmit(source.id, target.id);
            }
          }}
        >
          Submit
        </button>
      </div>
    </div>
  );
}

function TuptaCh() {
  const websocketProtocol =
    window.location.protocol === "https:" ? "wss:" : "ws:";
  const websocketAddress = `${websocketProtocol}//${window.location.host}/ws`;
  const ws = useRef<WebSocket | null>(null);

  const [availableRoutingNetworkNames, setAvailableRoutingNetworkNames] =
    useState<string[]>([]);

  const [routingNetworkName, setRoutingNetworkName] = useState<string | null>(
    null,
  );

  const [selecting, setSelecting] = useState<"source" | "target" | null>(null);
  const [pendingPoint, setPendingPoint] = useState<QueryPoint | null>(null);
  const [source, setSource] = useState<QueryPoint | null>(null);
  const [target, setTarget] = useState<QueryPoint | null>(null);

  const [path, setPath] = useState<Edge[] | null>(null);

  const [mapPoints, setMapPoints] = useState<Map<number, MapPoint>>(new Map());
  const [mapEdges, setMapEdges] = useState<Map<number, MapEdge>>(new Map());
  const pendingPointUpdates = useRef<Map<number, MapPoint>>(new Map());
  const pendingEdgeUpdates = useRef<Map<number, MapEdge>>(new Map());
  const mapPointsRef = useRef<Map<number, MapPoint>>(new Map());
  const mapEdgesRef = useRef<Map<number, MapEdge>>(new Map());

  function clear() {
    setSelecting(null);
    setPendingPoint(null);
    setSource(null);
    setTarget(null);
    setMapPoints(new Map());
    setMapEdges(new Map());
    setPath(null);
  }

  const [phase, setPhase] = useState<Phase>("SelectRoutingNetwork");

  const [numStepsInProgress, setNumStepsInProgress] = useState<number>(0);

  const pathLength = path?.reduce((acc, edge) => acc + edge.props.length, 0);

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
        switch (control_event.type) {
          case "AvailableRoutingNetworks":
            setAvailableRoutingNetworkNames(
              control_event.routing_network_names,
            );
            break;
          case "RoutingNetworkReady":
            setPhase("SelectAlgorithm");
            break;
          case "PreprocessingReady":
            setPhase("Preprocessing");
            break;
          case "PreprocessingDone":
            setPhase("SelectQuery");
            break;
          case "QueryReady":
            setPhase("Query");
            break;
          case "QueryDone":
            setPhase("QueryDone");
            setPath(control_event.path);
            break;
          case "ClosestVertexResponse":
            setPendingPoint(control_event);
            break;
          case "StepDone":
            setNumStepsInProgress((n) => n - 1);
            console.log(
              "StepDone, pending points:",
              pendingPointUpdates.current.size,
            );
            if (
              pendingPointUpdates.current.size > 0 ||
              pendingEdgeUpdates.current.size > 0
            ) {
              setMapPoints((prev) => {
                const next = new Map(prev);
                pendingPointUpdates.current.forEach((v, k) => next.set(k, v));
                pendingPointUpdates.current.clear();
                mapPointsRef.current = next;
                return next;
              });
              setMapEdges((prev) => {
                const next = new Map(prev);
                pendingEdgeUpdates.current.forEach((v, k) => next.set(k, v));
                pendingEdgeUpdates.current.clear();
                mapEdgesRef.current = next;
                return next;
              });
            }
            break;
        }
      } else if (server_event.type === "Algo") {
        const algo_event: AlgoEvent = server_event.event;
        const action = algo_event.action;
        console.log(action);

        switch (action.type) {
          case "HighlightVertex":
            console.log("adding pending point", action.vertex.id);
            pendingPointUpdates.current.set(action.vertex.id, {
              id: action.vertex.id,
              longitude: action.vertex.props.longitude,
              latitude: action.vertex.props.latitude,
              color: highlightColor(action.mode),
              radius: action.mode === "Source" ? 6 : 13,
            });
            console.log("adding", action.vertex.props);
            break;

          case "HighlightEdge":
            pendingEdgeUpdates.current.set(action.edge.id, {
              id: action.edge.id,
              path: action.edge.props.points.map((p) => [
                p.longitude,
                p.latitude,
              ]),
              color: highlightColor(action.mode),
              width: action.mode === "Source" ? 3 : 1,
            });
            break;
        }
      }
    };
    return () => ws.current?.close();
  }, []);

  function handleMapClick(latitude: number, longitude: number) {
    if (phase === "SelectQuery" && selecting !== null) {
      let latLng = { latitude, longitude };
      send({ type: "ClosestVertexRequest", name: selecting, lat_lng: latLng });
    }
  }

  function handleRoutingNetworkSelect(newRoutingNetworkName: string) {
    if (newRoutingNetworkName !== routingNetworkName) {
      clear();
      setRoutingNetworkName(newRoutingNetworkName);
      send({
        type: "SelectRoutingNetwork",
        routing_network_name: newRoutingNetworkName,
      });
      setPhase("RoutingNetworkSelected");
    }
  }

  function handleAlgorithmSelect(algorithmSelection: AlgorithmSelection) {
    clear();
    send({ type: "SelectAlgorithm", algorithm_selection: algorithmSelection });
    setPhase("AlgorithmSelected");
  }

  function handleQuerySelect(sourceId: number, targetId: number) {
    send({ type: "SelectQuery", source_id: sourceId, target_id: targetId });
    setPhase("QuerySelected");
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

        {phase === "Preprocessing" && (
          <>
            <h5>Control the preprocessing</h5>
            <Controls
              numStepsInProgress={numStepsInProgress}
              requestStep={requestStep}
              requestRunToCompletion={requestRunToCompletion}
            />
          </>
        )}

        {phaseOrder[phase] >= phaseOrder["SelectQuery"] && (
          <>
            <h4>Preprocessing is done</h4>
            {phase === "SelectQuery" && (
              <QuerySelector
                onSubmit={handleQuerySelect}
                source={source}
                setSource={setSource}
                target={target}
                setTarget={setTarget}
                pendingPoint={pendingPoint}
                setPendingPoint={setPendingPoint}
                selecting={selecting}
                setSelecting={setSelecting}
              />
            )}
            {phase === "Query" && (
              <>
                <h5>Control the query</h5>
                <Controls
                  numStepsInProgress={numStepsInProgress}
                  requestStep={requestStep}
                  requestRunToCompletion={requestRunToCompletion}
                />
              </>
            )}
            {phase === "QueryDone" && (
              <div>
                <h4>Query is done</h4>
                {path === null ? (
                  <h5>No path found!</h5>
                ) : (
                  <h5>Path found! {(pathLength! / 1000).toFixed(3)} km</h5>
                )}
                <button
                  className="btn-primary"
                  onClick={() => {
                    setPhase("SelectQuery");
                    clear();
                  }}
                >
                  Clear
                </button>
              </div>
            )}
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
          onMapClick={handleMapClick}
          mapPoints={mapPoints}
          mapEdges={mapEdges}
          path={path}
          phase={phase}
          source={source}
          target={target}
          pendingPoint={pendingPoint}
        />
      </MapContainer>
    </div>
  );
}

export default TuptaCh;
