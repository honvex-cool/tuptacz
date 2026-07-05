import { useEffect, useState } from "react"
import { apiGet, apiPost } from "../core/api"
import "../core/Common.css"
import "./JakDotuptam.css"

import { MapContainer, Polyline, TileLayer, Marker, Tooltip, CircleMarker, Popup } from 'react-leaflet'
import L from 'leaflet'

import { journeyArrivalTime, journeyDepartureTime, type Journey, type JourneyLeg, type JourneyStop, type RouteType, type Shape, type Stop, type Trip } from "./model"
import Select from "react-select"
import "dayjs/locale/pl";
import dayjs from "dayjs"

import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import type { Dayjs } from "dayjs"

import { ThemeProvider, createTheme } from "@mui/material/styles";

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'

import { faTrainTram, faBus } from '@fortawesome/free-solid-svg-icons'

const theme = createTheme({
    palette: {
        mode: "dark",
    },
});


function getStops() {
    return apiGet("/transit/stops").then(res => res.json())
}

function getShapes() {
    return apiGet("/transit/shapes").then(res => res.json())
}

async function getTrip(trip_id: number): Promise<Trip> {
    return apiGet(`/transit/trip/${trip_id}`).then(res => res.json())
}

const LINE_COLORS = [
    "#ff0211",
    "#bc6700",
    "#608500",
    "#008f00",
    "#008f74",
    "#008bc6"
]

const transparentIcon = L.divIcon({ className: '', iconSize: [0, 0] })

function LegPolyline(props: { leg: JourneyLeg, index: number }) {
    let positions = [];
    for (const stop of props.leg.stops) {
        positions.push([stop.position.latitude, stop.position.longitude])
    }
    const color = LINE_COLORS[props.index];
    const mid = positions[Math.floor(positions.length / 2)]
    let icon = props.leg.route_type == "Tram" ? faTrainTram : faBus;
    return (
        <>
            <Polyline key={props.index}
                positions={positions} color={color} weight={10}>
                <Marker position={mid} icon={transparentIcon}>
                    <Tooltip permanent direction="center" offset={[0, 0]}>
                        <FontAwesomeIcon icon={icon} />
                        {props.leg.route_name}
                    </Tooltip>
                </Marker>
            </Polyline>
            {props.leg.stops.map(stop => {
                return (
                    <CircleMarker center={[stop.position.latitude, stop.position.longitude]} radius={8} fill={true} fillOpacity={1} fillColor={"#eeeeee"} color={color}>
                        <Popup>
                            {stop.stop_name}
                        </Popup>
                    </CircleMarker>

                )
            })}
        </>
    )
}

function JourneyPolylines(props: { journey: Journey }) {
    if (!props.journey) {
        return (<></>)
    }

    return (
        <>
            {props.journey.legs.map((leg, index) => (<LegPolyline leg={leg} index={index} />))}
        </>
    )
}

function routeTypeIcon(routeType: RouteType) {
    let icon = routeType == "Tram" ? faTrainTram : faBus;
    return (<FontAwesomeIcon icon={icon} />)
}

const SECONDS_IN_DAY = 3600 * 24;

function formatTimeSeconds(time: number) {
    let hour = Math.floor(time % SECONDS_IN_DAY / 3600);
    let minute = Math.floor((time % 3600) / 60)

    return hour.toString().padStart(2, "0") + ":" + minute.toString().padStart(2, "0")
}

function formatDurationSeconds(duration: number) {
    let hours = Math.floor(duration / 3600)
    let minutes = Math.floor((duration % 3600) / 60)

    if (hours == 0) {
        return `${minutes} min`
    } else {
        if (minutes == 0) {
            return `${hours} h`
        } else {
            return `${hours} h ${minutes} min`
        }
    }
}

function RouteBadge(props: { route_name: string, route_type: RouteType, color: string }) {
    return (
        <div className={`journey-line`} style={{ backgroundColor: props.color }}>
            {routeTypeIcon(props.route_type)} {props.route_name}
        </div>
    )
}

function JourneySummaryLine(props: { leg: JourneyLeg, index: number }) {
    let leg = props.leg;
    let color = LINE_COLORS[props.index]
    return (
        <RouteBadge route_name={leg.route_name} route_type={leg.route_type} color={color}/>
    )
}

function pluralizeStops(stops) {
    if (stops === 1) {
        return "przystanek"
    } else if (10 < stops % 100 && stops % 100 < 20) {
        return "przystanków"
    } else if (stops % 10 === 2 || stops % 10 === 3 || stops % 10 === 4) {
        return "przystanki"
    } else {
        return "przystanków"
    }
}


function LegRow(props: { leg: JourneyLeg, color: string }) {
    const leg = props.leg;
    const color = props.color;

    const start = leg.stops[0];
    const end = leg.stops[leg.stops.length - 1];

    return (
        <div className="leg-details">
            <div className="leg-details-timeline">
                <div style={{ border: `1px solid ${color}`, width: "10px", height: "10px", flexShrink: 0}}></div>
                <div style={{ backgroundColor: color,  width: "1px", height: "10px", flexGrow: 1}}></div>
                <div style={{ border: `1px solid ${color}`, width: "10px", height: "10px", flexShrink: 0}}></div>
            </div>
            <div className="leg-details-content">
                <div> <div className="stop-name">{start.stop_name}</div> {formatTimeSeconds(start.arrival_time)} </div>
                <div style={{display: "flex", gap: "10px"}}> <div style={{ color: color }}>{routeTypeIcon(leg.route_type)} {leg.route_name}</div> <div>{leg.stops.length} {pluralizeStops(leg.stops.length)}</div> </div>
                <div>{formatTimeSeconds(end.arrival_time)} <div className="stop-name">{end.stop_name}</div></div>
            </div>
        </div>
    )
}

function TransferRow(props: { from: JourneyStop, to: JourneyStop, walkedDistance: number}) {
    return (
        <div className="transfer-details">
            <div className="transfer-details-timeline">
                <div style={{ borderLeft: "2px dashed #888", height: "20px" }}></div>
            </div>
            <div className="transfer-details-content">
                {props.walkedDistance}m pieszo
            </div>
        </div>
    )
}

function JourneyDetails(props: { journey: Journey }) {
    let rows = []
    for (let i = 0; i < props.journey.legs.length; i++) {
        const leg = props.journey.legs[i];
        rows.push((<LegRow leg={leg} color={LINE_COLORS[i]} />))
        if (i + 1 < props.journey.legs.length) {
            const nextLeg = props.journey.legs[i + 1];
            rows.push((<TransferRow from={leg.stops[leg.stops.length - 1]} to={nextLeg.stops[0]} walkedDistance={leg.walked_distance}/>))
        }
    }

    return (
        <div className="journey-details">
            {rows}
        </div>
    )
}

function JourneySummary(props: { journey: Journey, isActive: boolean, onClick }) {
    const journey = props.journey;
    if (!journey) {
        return (<></>)
    }

    let departureTime = journeyDepartureTime(journey);
    let arrivalTime = journeyArrivalTime(journey);
    let duration = arrivalTime - departureTime;

    let changeCount = journey.legs.length - 1

    let activeClass = props.isActive ? "active" : ""

    return (
        <div>
            <div className={`journey-summary ${activeClass}`} onClick={props.onClick}>
                <div className="journey-summary-top">
                    <div className="journey-duration text-primary">{formatDurationSeconds(duration)}</div>
                    <div>
                        <div>{formatTimeSeconds(departureTime)} - {formatTimeSeconds(arrivalTime)}</div>
                        <div>Przesiadki: {changeCount}</div>
                    </div>
                </div>
                <div className="journey-lines">
                    {journey.legs.map((leg, i) => (<JourneySummaryLine leg={leg} index={i} />))}
                </div>
            </div>
            {props.isActive ? <JourneyDetails journey={journey} /> : <></>}
        </div>
    )
}

export default function JakDotuptam() {
    const [shapes, setShapes] = useState<Shape[]>([]);
    const [stops, setStops] = useState<Stop[]>([]);
    const [start, setStart] = useState<Stop>();
    const [end, setEnd] = useState<Stop>();
    const [date, setDate] = useState<Dayjs | null>(dayjs())
    const [journeys, setJourneys] = useState<Journey[]>([]);

    const [activeJourney, setActiveJourney] = useState<Journey>(null);
    const [activeJourneyIndex, setActiveJourneyIndex] = useState<number>(null);

    useEffect(() => {
        getShapes().then(setShapes)
    }, [])


    useEffect(() => {
        getStops().then(setStops);
    }, []);

    function search() {
        console.log(start, end, date)

        apiPost("/transit/search", {
            start: start.id,
            end: end.id,
            departure_time: date.format("YYYY-MM-DDTHH:mm:ssZ")
        },
        )
            .then(res => res.json())
            .then(journeys => { setJourneys(journeys) })
    }

    return <div className="container">
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

            <ThemeProvider theme={theme}>
                <LocalizationProvider dateAdapter={AdapterDayjs}
                    adapterLocale="pl"

                >
                    <DateTimePicker
                        value={date}
                        onChange={(newValue) => {
                            setDate(newValue)
                        }}
                        ampm={false}
                    />
                </LocalizationProvider>
            </ThemeProvider>

            <button className="btn-primary" onClick={() => search()}>Szukaj</button>
            {
                journeys.map((journey, index) => (
                    <JourneySummary journey={journey} isActive={activeJourneyIndex == index} onClick={
                        () => { setActiveJourney(journey); setActiveJourneyIndex(index) }
                    } />
                ))
            }
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
            <JourneyPolylines journey={activeJourney} />

        </MapContainer>
    </div>
}
