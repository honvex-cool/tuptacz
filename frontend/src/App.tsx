import './App.css'

import { DeckOverlay } from '@deck.gl-community/leaflet';
import { MapView } from '@deck.gl/core';
import { GeoJsonLayer, ArcLayer, ScatterplotLayer, PathLayer } from '@deck.gl/layers';
import { useMap, MapContainer, TileLayer } from 'react-leaflet'
import { useEffect, useState } from 'react';

import 'leaflet/dist/leaflet.css';
import { setupWS } from './core/ws';

function GraphComponent({ nodes, edges, shortcuts }) {
  const map = useMap();

  useEffect(() => {
    const overlay = new DeckOverlay({
      views: null,
      layers: []
    });
    map.addLayer(overlay);


    function update() {
      overlay.setProps({
        layers: [
          new ScatterplotLayer({
            id: "nodes",
            data: nodes,
            getPosition: d => [d.lng, d.lat],
            getRadius: 2,
            getFillColor: d =>
              d.contracted ? [255, 80, 80] : [0, 120, 255]
          }),

          new PathLayer({
            id: "edges",
            data: edges,
            getPath: d => d.properties.map(p => {
              return [p.longitude, p.latitude]
            }
            ),
            getColor: [100, 100, 100],
            getWidth: 1
          }),

          new ArcLayer({
            id: "shortcuts",
            data: shortcuts,
            getSourcePosition: d => {
              const s = nodes[d.source];
              return [s.lng, s.lat];
            },
            getTargetPosition: d => {
              const t = nodes[d.target];
              return [t.lng, t.lat];
            },
            getWidth: 2,
            getSourceColor: [255, 140, 0],
            getTargetColor: [255, 0, 200]
          })
        ]
      });
    }

    update();

    return () => {
      map.removeLayer(overlay);
    };
  }, [map, nodes, edges, shortcuts]);

  return null;
}

function App() {
  const initialNodes = [
    { lat: 50.030283078531774, lng: 19.907621146592238 },
    { lat: 50.02778810326561, lng: 19.904820920606536 },
    { lat: 50.02629976583603, lng: 19.90315396250886 }
  ]

  const initialEdges = [{ source: 0, target: 1 }, { source: 1, target: 2 }]
  const initialShortcuts = [{ source: 0, target: 2 }]

  const [nodes, setNodes] = useState(initialNodes);
  const [edges, setEdges] = useState(initialEdges);
  const [shortcuts, setShortcuts] = useState(initialShortcuts);

  setupWS((e) => {
    if (e.action.type === "InitGraph") {
      console.log(e.action.edges[0])
      setNodes(e.action.vertices.map(v => {return { lat: v.latitude, lng: v.longitude }}))
      setEdges(e.action.edges)
    } else if (e.action.type === "AddShortcut") {
      const source = e.action.source
      const target = e.action.target

      setShortcuts(s =>
        [...s, { source: source, target: target }]
      )
    }
  })

  return <MapContainer
    center={[50.030641402999564, 19.906958054507893]} zoom={13}
    style={{ height: "100vh" }}>
    <TileLayer
      url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
      attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
    />
    <GraphComponent nodes={nodes} edges={edges} shortcuts={shortcuts} />
  </MapContainer>

}

export default App
