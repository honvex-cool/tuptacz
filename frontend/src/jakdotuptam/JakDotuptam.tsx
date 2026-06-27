import { useEffect, useState } from "react"
import { apiGet, apiPost } from "../core/ws"
import "./JakDotuptam.css"

import { MapContainer, Polyline, TileLayer, Marker, Tooltip } from 'react-leaflet'
import L from 'leaflet'

import { journeyArrivalTime, journeyDepartureTime, type Journey, type JourneyLeg, type RouteType, type Shape, type Stop, type Trip } from "./model"
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
function JourneyPolylines(props: { journey: Journey }) {

    if (!props.journey) {
        return (<></>)
    }

    let lines = []
    for (let i = 0; i < props.journey.legs.length; i++) {
        const leg = props.journey.legs[i]
        let positions = [];
        for (const stop of leg.stops) {
            console.log(stop)
            positions.push([stop.position.latitude, stop.position.longitude])
        }
        const color = LINE_COLORS[i];
        lines.push([color, positions, leg.route_name, leg.route_type])
    }

    console.log(lines)

    return (
        <>
            {
                lines.map(([color, positions, name, type], i) => {
                    const mid = positions[Math.floor(positions.length / 2)]
                    let icon = type == "Tram" ? faTrainTram : faBus;
                    return (<Polyline key={i}
                        positions={positions} color={color} weight={10}>
                        <Marker position={mid} icon={transparentIcon}>
                            <Tooltip permanent direction="center" offset={[0, 0]}>
                                <FontAwesomeIcon icon={icon} />
                                {name}
                            </Tooltip>
                        </Marker>
                    </Polyline>
                    )
                }
                )

            }
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

function JourneySummaryLine(props: { leg: JourneyLeg, index: number }) {
    let leg = props.leg;
    let color = LINE_COLORS[props.index]
    return (
        <div className={`journey-line line-type-${leg.route_type}`}
            style={{ backgroundColor: color }}
        >
            {routeTypeIcon(leg.route_type)} {leg.route_name}
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
            .then(journeys => { console.log("AAAA: ", journeys); setJourneys(journeys) })
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