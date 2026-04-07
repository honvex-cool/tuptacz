import reactLogo from './assets/react.svg'
import viteLogo from './assets/vite.svg'
import heroImg from './assets/hero.png'
import './App.css'

import { useEffect, useState, useRef } from 'react'
import cytoscape from 'cytoscape'
import CytoscapeComponent from "react-cytoscapejs"
import { sendStepMessage, setupWS } from './core/ws'

function App() {

  const cyRef = useRef<any>(null);

  const elements = [
    { data: { id: 'one' } },
    { data: { id: 'two' } },
    { data: { source: 'one', target: 'two' } }
  ];

  setupWS((e) => {
    const cy = cyRef.current
    if (e.eventType == "ADD_NODE") {
      cy.add([{ data: { id: e.nodeId } }])
      cy.layout({ name: "cose" }).run();
    }
    else if (e.eventType == "ADD_EDGE") {
      cy.add(
        {
          data: { source: e.startNodeId, target: e.endNodeId }
        }
      )
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
        elements={elements}
        layout={{ name: "grid" }}
        style={{ width: "100%", height: "100%" }}
      />
    </div>
  )
}

export default App
