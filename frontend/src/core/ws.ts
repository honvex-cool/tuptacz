export const ws = new WebSocket("ws://localhost:3000/ws");

export interface Edge {
    id: number
    end_id: number
    properties: any
}

export interface Vertex {
    id: number
    properties: any
    edges: Edge[]
}

export type Graph = Vertex[];

export interface InitGraphAction {
    type: "InitGraph"
    graph: Graph
}

export interface AddNodeAction {
    type: "AddVertex"
    id: number
}

export interface AddEdgeAction {
    type: "AddEdge"
    start_id: number
    end_id: number
}

export interface HighlightVertexAction {
    type: "HighlightVertex"
    id: number
    mode: "Awaiting" | "Visited"
}

export interface HighlightEdgeAction {
    type: "HighlightEdge"
    id: number
    mode: string
}

export type Action = 
    InitGraphAction
    | AddNodeAction 
    | AddEdgeAction 
    | HighlightVertexAction 
    | HighlightEdgeAction

export interface Event {
    action: Action,
    comment: string
}

export function setupWS(onMessage: (e: Event) => void) {
    ws.onmessage = (event) => {
        const data = JSON.parse(event.data)
        console.log(data)
        onMessage(data)
    }
}

export function sendStepMessage() {
    console.log("sending step")
    ws.send(JSON.stringify({ "type": "STEP" }))
}