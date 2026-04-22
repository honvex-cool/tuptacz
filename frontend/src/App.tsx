import './App.css'

import { useEffect, useState, useRef } from 'react'
import CytoscapeComponent from "react-cytoscapejs"
import { sendStepMessage, setupWS } from './core/ws'

function App() {

  const cyRef = useRef<any>(null);

  setupWS((event) => {
    const cy = cyRef.current
    const action = event.action;
    switch (action.type) {
      case "InitGraph": {
        const vertices = action.graph.map(v => { return { data: { id: v.id, label: "" } } })
        const edges = action.graph.flatMap(v => v.edges.map(e => { return { data: { source: v.id, target: e.end_id, label: e.properties } } }))
        console.log(vertices, edges)
        cy.add(vertices)
        cy.add(edges)
        cy.layout({ name: "cose" }).run();
        cy.style([
          {
            selector: 'node',
            style: { 'background-color': '#666' }
          },
          {
            selector: 'edge',
            style: {
              'line-color': '#ccc',
              'curve-style': 'bezier',
              'target-arrow-shape': 'triangle',
              'target-arrow-color': '#ccc'
            }
          },
          {
            selector: 'edge[label]',
            style: {
              'label': 'data(label)',
              'color': '#eee',
              'text-background-color': '#222',
              'text-background-opacity': 0.75,
              'text-background-padding': '2px',
              'target-arrow-shape': 'triangle',
              'target-arrow-color': '#ccc'
            }
          }
        ])
        break
      }
      case "AddVertex": {
        cy.add([{ data: { id: action.id } }])
        cy.layout({ name: "cose" }).run();
        break
      }
      case "AddEdge": {
        cy.add({
          data: { source: action.start_id, target: action.end_id }
        })
        break
      }
      case "HighlightVertex": {
        const node = cy.getElementById(action.id.toString());
        console.log(action.mode)
        if (action.mode == "Awaiting") {
          node.style("background-color", "red")
        } else if (action.mode == "Visited") {
          node.style("background-color", "black")
        }
        break
      }
      case "HighlightEdge": { break }

    }
  })

  return (
    <div style={{ width: "100%", height: "100vh" }}>
      <div className="btns">
        <div className="btn" onClick={() => sendStepMessage()}>
          Step
        </div>
      </div>
      <CytoscapeComponent
        cy={(cy) => {
          cyRef.current = cy
        }}
        elements={[]}
        layout={{ name: "grid" }}
        style={{ width: "100%", height: "100%" }}
      />
    </div>
  )
}

export default App
