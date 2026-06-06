const websocketProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';

const ws = new WebSocket(
    `${websocketProtocol}//${window.location.host}/ws`
);

export interface Coords {
    latitude: number
    longitude: number
}

export interface Edge {
    source: number,
    target: number,
    properties: Coords[]
}

export interface Vertex {
    latitude: number
    longitude: number
}

export interface InitGraphAction {
    type: "InitGraph"
    vertices: Vertex[],
    edges: Edge[]
}

export interface AddShortcutAction {
    type: "AddShortcut"
    source: number
    target: number
}

export type Action =
    InitGraphAction
    | AddShortcutAction

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

export function apiGet(path: string) {
    return fetch(`${window.location.protocol}//${window.location.host}/api${path}`)
}

export function apiPost(path: string, payload: any) {
    return fetch(
        `${window.location.protocol}//${window.location.host}/api${path}`, {
        method: "POST",
        body: JSON.stringify(payload),
        headers: {
            "Content-type": "application/json; charset=UTF-8"
        }
    }
    )
}
