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

export type HighlightMode = "Visited" | "Awaiting" | "Source";

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

export interface Progress {
  type: "Progress";
  current: number;
  total: number;
}

export type GraphAction = HighlightVertex | HighlightEdge | Progress;

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

export interface RunPreprocessingToCompletion {
  type: "RunPreprocessingToCompletion";
}

export interface SelectQuery {
  type: "SelectQuery";
  source_id: number;
  target_id: number;
}

export interface StepQuery {
  type: "StepQuery";
}

export interface RunQueryToCompletion {
  type: "RunQueryToCompletion";
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
  | RunPreprocessingToCompletion
  | SelectQuery
  | StepQuery
  | RunQueryToCompletion
  | ClosestVertexRequest;
