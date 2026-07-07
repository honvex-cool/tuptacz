import type { GeoJsonObject } from "geojson";

export interface LatLng {
  latitude: number;
  longitude: number;
}

export interface Vertex {
  id: number;
  props: LatLng;
}

export interface Road {
  points: LatLng[];
  length: number;
}

export interface Edge {
  end: Vertex;
  start: Vertex;
  id: number;
  props: Road;
}

export interface PriorityParts {
  e: number;
  s: number;
  d: number;
  o: number;
  q: number;
}

export type HighlightMode = "Visited" | "Awaiting" | "Source";

export type HighlightDescription =
  "Contraction" | "LazyUpdate" | "UpdateInGlobal" | "Short" | "Long";

export interface HighlightVertex {
  type: "HighlightVertex";
  vertex: Vertex;
  mode: HighlightMode;
}

export interface HighlightEdge {
  type: "HighlightEdge";
  edge: Edge;
  mode: HighlightMode;
}

export interface Contraction {
  type: "Contraction";
  vertex: Vertex;
  shortcuts: [number, Edge, Edge][];
}

export interface LazyUpdate {
  type: "LazyUpdate";
  vertex: Vertex;
}

export interface UpdateInGlobal {
  type: "UpdateInGlobal";
  vertex: Vertex;
  coefficients: PriorityParts;
  terms: PriorityParts;
  priority: number;
}

export interface GlobalUpdateTriggered {
  type: "GlobalUpdateTriggered";
  ratio: number;
  allowed_ratio: number;
  time_since_global_update: number;
}

export interface QuerySummary {
  type: "QuerySummary";
  num_settled_vertices: number;
  num_inspected_edges: number;
}

export interface ContractionSummary {
  type: "ContractionSummary";
  stats: {
    num_steps: number;
    num_contractions: number;
    num_shortcuts: number;
    num_lazy_updates: number;
    num_global_updates: number;
  };
}

export interface Interrupt {
  type: "Interrupt";
}

export interface Progress {
  type: "Progress";
  current: number;
  total: number;
}

export type GraphAction =
  | HighlightVertex
  | HighlightEdge
  | Contraction
  | LazyUpdate
  | UpdateInGlobal
  | GlobalUpdateTriggered
  | QuerySummary
  | ContractionSummary
  | Interrupt
  | Progress;

export interface AlgoEvent {
  action: GraphAction;
  comment: string;
}

export interface Algo {
  type: "Algo";
  event: AlgoEvent;
}

export interface AvailableRoutingNetworks {
  type: "AvailableRoutingNetworks";
  routing_network_names: string[];
}

export interface RoutingNetworkReady {
  type: "RoutingNetworkReady";
  num_vertices: number;
  num_edges: number;
  polygon: GeoJsonObject;
}

export interface PreprocessingReady {
  type: "PreprocessingReady";
}

export interface PreprocessingDone {
  type: "PreprocessingDone";
}

export interface QueryReady {
  type: "QueryReady";
}

export interface QueryDone {
  type: "QueryDone";
  path: Edge[] | null;
}

export interface StepDone {
  type: "StepDone";
}

export interface ClosestVertexResponse {
  type: "ClosestVertexResponse";
  name: string;
  lat_lng: LatLng;
  id: number;
}

export type ControlEvent =
  | AvailableRoutingNetworks
  | RoutingNetworkReady
  | PreprocessingReady
  | PreprocessingDone
  | QueryReady
  | QueryDone
  | StepDone
  | ClosestVertexResponse;

export interface Control {
  type: "Control";
  event: ControlEvent;
}

export type ServerEvent = Control | Algo;

export interface Dijkstra {
  type: "Dijkstra";
  is_bidirectional: boolean;
}

export interface AStar {
  type: "AStar";
  is_bidirectional: boolean;
}

export interface ContractionHierarchies {
  type: "ContractionHierarchies";
}

export type AlgorithmSelection = Dijkstra | AStar | ContractionHierarchies;

export interface SelectRoutingNetwork {
  type: "SelectRoutingNetwork";
  routing_network_name: string;
}

export interface SelectAlgorithm {
  type: "SelectAlgorithm";
  algorithm_selection: AlgorithmSelection;
}

export interface StepPreprocessing {
  type: "StepPreprocessing";
}

export interface RunPreprocessingFreely {
  type: "RunPreprocessingFreely";
}

export interface SelectQuery {
  type: "SelectQuery";
  source_id: number;
  target_id: number;
}

export interface StepQuery {
  type: "StepQuery";
}

export interface RunQueryFreely {
  type: "RunQueryFreely";
}

export interface ClosestVertexRequest {
  type: "ClosestVertexRequest";
  name: string;
  lat_lng: LatLng;
}

export type FrontendEvent =
  | SelectRoutingNetwork
  | SelectAlgorithm
  | StepPreprocessing
  | RunPreprocessingFreely
  | SelectQuery
  | StepQuery
  | RunQueryFreely
  | ClosestVertexRequest;
