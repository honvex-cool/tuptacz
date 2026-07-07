import "../core/Common.css";
import "./TuptaCh.css";
import { DeckOverlay } from "@deck.gl-community/leaflet";
import { PathLayer, ScatterplotLayer } from "@deck.gl/layers";
import {
  useMap,
  MapContainer,
  TileLayer,
  useMapEvents,
  GeoJSON,
  CircleMarker,
  Tooltip,
} from "react-leaflet";
import { useEffect, useRef, useState } from "react";
import "leaflet/dist/leaflet.css";
import type {
  AlgoEvent,
  AlgorithmSelection,
  ControlEvent,
  Edge,
  FrontendEvent,
  HighlightDescription,
  HighlightMode,
  LatLng,
  ServerEvent,
  Vertex,
} from "./presentation";
import Slider from "@mui/material/Slider";
import Select from "react-select";
import type { GeoJsonObject } from "geojson";
import L from "leaflet";

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

type RoutingNetwork = {
  numVertices: number;
  numEdges: number;
  polygon: GeoJsonObject;
};

type MapPoint = {
  longitude: number;
  latitude: number;
  color: [number, number, number];
  radius: number;
};

type MapEdge = {
  path: [number, number][];
  color: [number, number, number];
  width: number;
};

type Progress = {
  current: number;
  total: number;
};

function highlightColor(
  mode: HighlightMode | HighlightDescription,
): [number, number, number] {
  switch (mode) {
    case "Visited":
      return [255, 80, 80];
    case "Awaiting":
      return [0, 120, 255];
    case "Source":
      return [0, 255, 0];
    case "Contraction":
      return [255, 0, 0];
    case "LazyUpdate":
      return [0, 0, 255];
    case "UpdateInGlobal":
      return [0, 255, 0];
    case "Long":
      return [0, 0, 255];
    case "Short":
      return [255, 0, 0];
  }
}

function modeComment(mode: HighlightMode) {
  switch (mode) {
    case "Source":
      return "is a source";
    case "Awaiting":
      return "enqueued";
    case "Visited":
      return "settled";
  }
}

function InfoComponent({ algoEvent }: { algoEvent: AlgoEvent }) {
  const action = algoEvent.action;
  switch (action.type) {
    case "HighlightVertex":
      const [r, g, b] = highlightColor(action.mode);
      return (
        <>
          Vertex{" "}
          <span style={{ color: `rgb(${r}, ${g}, ${b})` }}>
            {modeComment(action.mode)}
          </span>
        </>
      );
    case "HighlightEdge":
      return <></>;
    case "Contraction":
      return (
        <>
          <div>
            <span style={{ color: "blue" }}>Contraction</span>
          </div>
          <div className="details-box">
            added {action.shortcuts.length} shortcuts
          </div>
        </>
      );
    case "LazyUpdate":
      return (
        <>
          <div>
            <span style={{ color: "yellow" }}>Updated (lazy)</span>
          </div>
        </>
      );
    case "UpdateInGlobal":
      return (
        <>
          <div>
            <span style={{ color: "blue" }}>Updated (global):</span>
          </div>
          <div className="details-box">
            <div>
              E: {action.coefficients.e} * {action.terms.e}
            </div>
            <div>
              S: {action.coefficients.s} * {action.terms.s}
            </div>
            <div>
              D: {action.coefficients.d} * {action.terms.d}
            </div>
            <div>
              O: {action.coefficients.o} * {action.terms.o}
            </div>
            <div>
              Q: {action.coefficients.q} * {action.terms.q}
            </div>
          </div>
        </>
      );
    case "GlobalUpdateTriggered":
      return (
        <div>
          <span style={{ color: "green" }}>Global update triggered</span>
        </div>
      );
    case "QuerySummary":
      return (
        <>
          <div>
            <span style={{ color: "green" }}>Query phase complete:</span>
          </div>
          <div className="details-box">
            <div>{action.num_settled_vertices} settled vertices</div>
            <div>{action.num_inspected_edges} inspected edges</div>
          </div>
        </>
      );
    case "ContractionSummary":
      return (
        <>
          <div>
            <span style={{ color: "green" }}>Contraction phase complete:</span>
          </div>
          <div className="details-box">
            <div>{action.stats.num_steps} total steps</div>
            <div>
              ({action.stats.num_contractions} contractions)
            </div>
            <div>{action.stats.num_shortcuts} shortcuts added</div>
            <div>{action.stats.num_lazy_updates} lazy updates</div>
            <div>{action.stats.num_global_updates} global updates</div>
          </div>
        </>
      );
    case "Interrupt":
      return (
        <>
          <div>
            <span style={{ color: "orange" }}>Free run interrupted:</span>
          </div>
          <div className="details-box">{algoEvent.comment}</div>
        </>
      );
    case "Progress":
      return <></>;
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

function PolygonComponent({ polygon }: { polygon: GeoJsonObject }) {
  const map = useMap();

  useEffect(() => {
    const layer = L.geoJSON(polygon);
    map.fitBounds(layer.getBounds());
  }, [map, polygon]);

  return (
    <GeoJSON
      data={polygon}
      style={{ fillOpacity: 0.1, weight: 1, color: "#3388ff" }}
    />
  );
}

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
  isRunningFreely,
  requestStep,
  requestRunFreely,
}: {
  numStepsInProgress: number;
  isRunningFreely: boolean;
  requestStep: () => void;
  requestRunFreely: () => void;
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

  const isAutoplay = autoplaySpeed !== null;

  return (
    <div className="controls">
      <button
        className="btn-primary"
        disabled={isRunningFreely || isAutoplay || numStepsInProgress > 0}
        onClick={() => requestStep()}
      >
        Step
      </button>
      <div>
        <button
          className="btn-primary"
          disabled={isRunningFreely}
          onClick={() => {
            if (isAutoplay) {
              stopAutoplay();
            } else {
              startAutoplay(autoplaySpeed ?? 1);
            }
          }}
        >
          {isAutoplay ? "Manual" : "Autoplay"}
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
        {autoplaySpeed}
      </label>
      <div>
        <button
          className="btn-primary"
          disabled={isRunningFreely}
          onClick={() => {
            stopAutoplay();
            requestRunFreely();
          }}
        >
          Run freely
        </button>
      </div>
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
          <div>Configure algorithm</div>
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
      Select Query Points
      {(pendingPoint === null || selecting === "target") && (
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
      )}
      {selecting === "source" && pendingPoint !== null && (
        <div>
          <button className="btn-primary" onClick={acceptPending}>
            Accept
          </button>
          <span>
            Nearest: ({pendingPoint.lat_lng.latitude.toFixed(4)},{" "}
            {pendingPoint.lat_lng.longitude.toFixed(4)})
          </span>
        </div>
      )}
      {(pendingPoint === null || selecting === "source") && (
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
      )}
      {selecting === "target" && pendingPoint !== null && (
        <div>
          <button className="btn-primary" onClick={acceptPending}>
            Accept
          </button>
          <span>
            Nearest: ({pendingPoint.lat_lng.latitude.toFixed(4)},{" "}
            {pendingPoint.lat_lng.longitude.toFixed(4)})
          </span>
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
  const ws = useRef<WebSocket | null>(null);

  const [availableRoutingNetworkNames, setAvailableRoutingNetworkNames] =
    useState<string[]>([]);

  const [selectedRoutingNetworkName, setSelectedRoutingNetworkName] = useState<
    string | null
  >(null);
  const [routingNetwork, setRoutingNetwork] = useState<RoutingNetwork | null>(
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

  const [phase, setPhase] = useState<Phase>("SelectRoutingNetwork");
  const [isRunningFreely, setIsRunningFreely] = useState<boolean>(false);
  const [numStepsPending, setNumStepsPending] = useState<number>(0);

  const [progress, setProgress] = useState<Progress | null>(null);
  const [algoEvents, setAlgoEvents] = useState<AlgoEvent[]>([]);

  function clearAlgoEvents() {
    setAlgoEvents([]);
  }

  function addAlgoEvent(algoEvent: AlgoEvent) {
    setAlgoEvents((prev) => [...prev, algoEvent]);
  }

  function clearQuerySelection() {
    setSelecting(null);
    setPendingPoint(null);
    setSource(null);
    setTarget(null);
  }

  function clearMapDisplay() {
    pendingPointUpdates.current.clear();
    mapPointsRef.current.clear();
    setMapPoints(new Map());

    pendingEdgeUpdates.current.clear();
    mapEdgesRef.current.clear();
    setMapEdges(new Map());
  }

  function clear() {
    clearAlgoEvents();
    clearQuerySelection();
    clearMapDisplay();
    setPath(null);
  }

  function send(event: FrontendEvent) {
    console.log(`sending ${event.type}`);
    ws.current?.send(JSON.stringify(event));
  }

  function addPendingVertex(
    vertex: Vertex,
    mode: HighlightMode | HighlightDescription,
  ) {
    pendingPointUpdates.current.set(vertex.id, {
      longitude: vertex.props.longitude,
      latitude: vertex.props.latitude,
      color: highlightColor(mode),
      radius: 13,
    });
  }

  function addPendingPolyline(
    id: number,
    points: LatLng[],
    mode: HighlightMode | HighlightDescription,
  ) {
    pendingEdgeUpdates.current.set(id, {
      path: points.map((p) => [p.longitude, p.latitude]),
      color: highlightColor(mode),
      width: 3,
    });
  }

  function addPendingEdge(
    edge: Edge,
    mode: HighlightMode | HighlightDescription,
  ) {
    const points =
      edge.props.points.length === 0
        ? edge.props.points
        : [edge.start.props, edge.end.props];
    addPendingPolyline(edge.id, points, mode);
  }

  function requestStep() {
    clearAlgoEvents();
    setNumStepsPending((n) => n + 1);
    const type = phase === "Preprocessing" ? "StepPreprocessing" : "StepQuery";
    send({ type });
  }

  function requestRunFreely() {
    clearAlgoEvents();
    const type =
      phase === "Preprocessing" ? "RunPreprocessingFreely" : "RunQueryFreely";
    send({ type });
  }

  function handleMapClick(latitude: number, longitude: number) {
    if (phase === "SelectQuery" && selecting !== null) {
      let latLng = { latitude, longitude };
      send({ type: "ClosestVertexRequest", name: selecting, lat_lng: latLng });
    }
  }

  function handleRoutingNetworkSelect(newRoutingNetworkName: string) {
    if (newRoutingNetworkName !== selectedRoutingNetworkName) {
      clear();
      setSelectedRoutingNetworkName(newRoutingNetworkName);
      setRoutingNetwork(null);
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

  useEffect(() => {
    const websocketProtocol =
      window.location.protocol === "https:" ? "wss:" : "ws:";
    const websocketAddress = `${websocketProtocol}//${window.location.host}/ws`;
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
            setRoutingNetwork({
              numVertices: control_event.num_vertices,
              numEdges: control_event.num_edges,
              polygon: control_event.polygon,
            });
            setPhase("SelectAlgorithm");
            setProgress(null);
            break;

          case "PreprocessingReady":
            setPhase("Preprocessing");
            setProgress(null);
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
            setNumStepsPending((n) => n - 1);
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
        addAlgoEvent(algo_event);
        const action = algo_event.action;
        console.log(`action.type: ${action.type}`);

        switch (action.type) {
          case "HighlightVertex":
            addPendingVertex(action.vertex, action.mode);
            break;

          case "HighlightEdge":
            addPendingEdge(action.edge, action.mode);
            break;

          case "Contraction":
            clearMapDisplay();
            addPendingVertex(action.vertex, "Contraction");
            action.shortcuts.forEach((shortcut) => {
              const [id, first, second] = shortcut;
              addPendingEdge(first, "Long");
              addPendingEdge(second, "Long");
              let shortcutPolyline = [first.start.props, second.end.props];
              addPendingPolyline(id, shortcutPolyline, "Short");
            });
            break;

          case "LazyUpdate":
            clearMapDisplay();
            addPendingVertex(action.vertex, "LazyUpdate");
            break;

          case "UpdateInGlobal":
            clearMapDisplay();
            addPendingVertex(action.vertex, "UpdateInGlobal");
            break;

          case "GlobalUpdateTriggered":
            clearMapDisplay();
            break;

          case "QuerySummary":
            break;

          case "ContractionSummary":
            break;

          case "Interrupt":
            setIsRunningFreely(false);
            break;

          case "Progress":
            setProgress(action);
            break;
        }
      }
    };
    return () => ws.current?.close();
  }, []);

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
          <>
            {routingNetwork?.numVertices ?? 0} vertices,{" "}
            {routingNetwork?.numEdges ?? 0} edges
            <AlgorithmSelector onSelect={handleAlgorithmSelect} />
          </>
        )}

        {phase === "Preprocessing" && (
          <>
            Control the preprocessing
            <Controls
              numStepsInProgress={numStepsPending}
              isRunningFreely={isRunningFreely}
              requestStep={requestStep}
              requestRunFreely={requestRunFreely}
            />
          </>
        )}

        {phaseOrder[phase] >= phaseOrder["SelectQuery"] && (
          <>
            Preprocessing is done
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
                Control the query
                <Controls
                  numStepsInProgress={numStepsPending}
                  isRunningFreely={isRunningFreely}
                  requestStep={requestStep}
                  requestRunFreely={requestRunFreely}
                />
              </>
            )}
            {phase === "QueryDone" && (
              <div>
                Query is done
                {path === null ? (
                  <div>No path found!</div>
                ) : (
                  <div>
                    Path found!{" "}
                    {(
                      path.reduce((acc, edge) => acc + edge.props.length, 0) /
                      1000
                    ).toFixed(3)}{" "}
                    km ({path.length} edges)
                  </div>
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

        {progress !== null && progress.current < progress.total && (
          <>
            <div>
              <progress value={progress.current} max={progress.total} />
            </div>
            <div>
              {((progress.current / progress.total) * 100).toFixed(0)} % (
              {progress.current} / {progress.total})
            </div>
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
        {routingNetwork !== null && (
          <PolygonComponent polygon={routingNetwork.polygon} />
        )}
        {source && (
          <CircleMarker
            center={[source.lat_lng.latitude, source.lat_lng.longitude]}
            radius={8}
            pathOptions={{ color: "green", fillColor: "green", fillOpacity: 1 }}
          >
            <Tooltip permanent>Source</Tooltip>
          </CircleMarker>
        )}
        {target && (
          <CircleMarker
            center={[target.lat_lng.latitude, target.lat_lng.longitude]}
            radius={8}
            pathOptions={{ color: "blue", fillColor: "blue", fillOpacity: 1 }}
          >
            <Tooltip permanent>Target</Tooltip>
          </CircleMarker>
        )}
        {pendingPoint && (
          <CircleMarker
            center={[
              pendingPoint.lat_lng.latitude,
              pendingPoint.lat_lng.longitude,
            ]}
            radius={8}
            pathOptions={{
              color: "orange",
              fillColor: "orange",
              fillOpacity: 1,
            }}
          >
            <Tooltip permanent>Pending</Tooltip>
          </CircleMarker>
        )}
      </MapContainer>

      <div className="info-box">
        {algoEvents.map((algoEvent) => (
          <div>
            <InfoComponent algoEvent={algoEvent} />
          </div>
        ))}
      </div>
    </div>
  );
}

export default TuptaCh;
