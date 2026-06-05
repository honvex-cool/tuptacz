import { useEffect, useState } from "react"
import { apiGet } from "../core/ws"
import "./JakDotuptam.css"

import { MapContainer, Polyline, TileLayer, Marker, Popup } from 'react-leaflet'
import type { Shape, Stop } from "./gtfs"

function getStops() {
    return apiGet("/gtfs/stops").then(res => res.json())
}

function getShapes() {
    return apiGet("/gtfs/shapes").then(res => res.json())
}

export default function JakDotuptam() {
    const [shapes, setShapes] = useState<Shape[]>([]);
    const [stops, setStops] = useState<Stop[]>([]);

    useEffect(() => {
        getShapes().then(setShapes)
    }, [])


    useEffect(() => {
        getStops().then(setStops);
    }, []);

    return <div id="jakdotuptam-container">
        <div className="route-search">
            <input type="text" placeholder="Skąd" />
            <input type="text" placeholder="Dokąd" />
            <button className="btn-primary">Szukaj</button>
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

            {shapes.map((shape) => (
                <Polyline
                    key={shape.id}
                    positions={shape.points.map((p) => [p.latitude, p.longitude])}
                    color="blue"
                />
            ))}

            {stops.map((stop) => (
                <Marker
                    key={stop.id}
                    position={[stop.point.latitude, stop.point.longitude]}
                >
                    <Popup>
                        <b>{stop.name}</b>
                        <br />
                        {stop.code && <span>Code: {stop.code}</span>}
                    </Popup>
                </Marker>
            ))}
        </MapContainer>
    </div>
}