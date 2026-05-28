import "./JakDotuptam.css"

import { useMap, MapContainer, TileLayer } from 'react-leaflet'

export default function JakDotuptam() {
    return <div id="jakdotuptam-container">
        <div>
            Menu with stuff
        </div>
        <MapContainer
            center={[50.030641402999564, 19.906958054507893]} zoom={13}
            style={{ height: "100vh" }}
            className="map-container"
            >
            <TileLayer
                url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
                attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
            />
        </MapContainer>
    </div>
}