export const ws = new WebSocket("ws://localhost:3000/ws");

export interface AddNodeEvent {
    eventType: "ADD_NODE"
    nodeId: string
}

export interface AddEdgeEvent {
    eventType: "ADD_EDGE"
    startNodeId: string
    endNodeId
}

export type Event = AddNodeEvent | AddEdgeEvent

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