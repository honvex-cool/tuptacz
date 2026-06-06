import { useEffect, useState } from "react"
import { apiGet, apiPost } from "../core/ws"
import "./JakDotuptam.css"

import { MapContainer, Polyline, TileLayer, Marker, Popup } from 'react-leaflet'
import type { Shape, Stop } from "./gtfs"
import Select from "react-select"

function getStops() {
    return apiGet("/transit/stops").then(res => res.json())
}

function getShapes() {
    return apiGet("/transit/shapes").then(res => res.json())
}

export default function JakDotuptam() {
    const [shapes, setShapes] = useState<Shape[]>([]);
    const [stops, setStops] = useState<Stop[]>([]);
    const [start, setStart] = useState<Stop>();
    const [end, setEnd] = useState<Stop>();

    useEffect(() => {
        getShapes().then(setShapes)
    }, [])


    useEffect(() => {
        getStops().then(setStops);
    }, []);

    function search() {
        apiPost("/transit/search", {
            start: start.id,
            end: end.id
        },
    
    )
    }

    return <div id="jakdotuptam-container">
        <div className="route-search">
            <Select options={stops.map(stop => {
                return {
                    value: stop,
                    label: stop.name
                }
            })}
                onChange={(newValue) => {
                    console.log(newValue)
                    setStart(newValue.value)
                }}
            />
            <Select options={stops.map(stop => {
                return {
                    value: stop,
                    label: stop.name
                }
            })}
                onChange={(newValue) => {
                    console.log(newValue)
                    setEnd(newValue.value)
                }}
            />
            <button className="btn-primary" onClick={() => search()}>Szukaj</button>
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

            {
                start && (
                    <Marker
                        position={[start.position.latitude, start.position.longitude]}
                        title="start"
                    />
                )
            }
            {
                end && (
                    <Marker
                        position={[end.position.latitude, end.position.longitude]}
                        title="koniec"
                    />
                )
            }


        </MapContainer>
    </div>
}