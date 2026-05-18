import './App.css'

import { DeckOverlay } from '@deck.gl-community/leaflet';
import { MapView } from '@deck.gl/core';
import { GeoJsonLayer, ArcLayer } from '@deck.gl/layers';
import { useMap, MapContainer, TileLayer } from 'react-leaflet'
import { useEffect } from 'react';

import 'leaflet/dist/leaflet.css';

function Graph() {
  // source: Natural Earth http://www.naturalearthdata.com/ via geojson.xyz
  const AIR_PORTS =
    'https://d2ad6b4ur7yvpq.cloudfront.net/naturalearth-3.3.0/ne_10m_airports.geojson';

  // Create map
  const map = useMap();
  useEffect(() => {
    // Add deck.gl overlay
    const deckOverlay = new DeckOverlay({
      views: [
        new MapView({ repeat: true }),
      ],
      layers: [
        new GeoJsonLayer({
          id: 'airports',
          data: AIR_PORTS,
          // Styles
          filled: true,
          pointRadiusMinPixels: 2,
          pointRadiusScale: 2000,
          getPointRadius: (f) => 11 - f.properties.scalerank,
          getFillColor: [200, 0, 80, 180],
          // Interactive props
          pickable: true,
          autoHighlight: true,
          onClick: (info) =>
            // eslint-disable-next-line
            info.object && alert(`${info.object.properties.name} (${info.object.properties.abbrev})`)
        }),
        new ArcLayer({
          id: 'arcs',
          data: AIR_PORTS,
          dataTransform: (d: any) => d.features.filter((f: any) => f.properties.scalerank < 4),
          // Styles
          getSourcePosition: () => [-0.4531566, 51.4709959], // London
          getTargetPosition: (f) => f.geometry.coordinates,
          getSourceColor: [0, 128, 200],
          getTargetColor: [200, 0, 80],
          getWidth: 1
        })
      ],
      getTooltip: (info) => info.object && info.object.properties.name
    });
    map.addLayer(deckOverlay);
  });

  return null;
}

function App() {
  return <MapContainer
    center={[51.505, -0.09]} zoom={13}
    style={{ height: "100vh" }}>
    <TileLayer
      url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
      attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
    />
    <Graph />
  </MapContainer>

}

export default App
