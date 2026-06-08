import { useEffect, useState } from "react"
import { apiGet, apiPost } from "../core/ws"
import "./JakDotuptam.css"

import { MapContainer, Polyline, TileLayer, Marker, Popup, Tooltip } from 'react-leaflet'
import L from 'leaflet'

import type { Journey, JourneyStep, Shape, Stop, Trip } from "./gtfs"
import Select from "react-select"
import "dayjs/locale/pl";
import dayjs from "dayjs"

import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import type { Dayjs } from "dayjs"

import { ThemeProvider, createTheme } from "@mui/material/styles";

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


type ValuePiece = Date | null;

type Value = ValuePiece | [ValuePiece, ValuePiece];

type EnrichedStep = {
    trip: Trip,
    step: JourneyStep
}
type EnrichedJourney = {
    steps: EnrichedStep[]
}

async function enrichJourney(journey: Journey) {
    const steps = await Promise.all(
        journey.steps.map(async s => ({
            trip: await getTrip(s.trip_id),
            step: s
        }))
    )

    return { steps }
}

const transparentIcon = L.divIcon({ className: '', iconSize: [0, 0] })
function JourneyPolylines(props: { journey: EnrichedJourney }) {
    const colors = [
        "#48d4af",
        "#1db7ab",
        "#089aa0",
        "#1d7e8e",
        "#2b6275",
        "#2f4858"
    ]

    if (!props.journey) {
        return (<></>)
    }

    let lines = []
    for (let i = 0; i < props.journey.steps.length; i++) {
        const step = props.journey.steps[i]
        let positions = [];
        for (let j = step.step.start_stop_idx; j <= step.step.end_stop_idx; j++) {
            let stop = step.trip.stops[j];
            positions.push([stop.position.latitude, stop.position.longitude])
        }
        const color = colors[i];
        lines.push([color, positions, step.trip.route.short_name])
    }

    console.log(lines)

    return (
        <>
            {
                lines.map(([color, positions, name], i) => {
                    const mid = positions[Math.floor(positions.length / 2)]
                    return (<Polyline key={i}
                        positions={positions} color={color} weight={10}>
                        <Marker position={mid} icon={transparentIcon}>
                            <Tooltip permanent direction="center" offset={[0, 0]}>
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

export default function JakDotuptam() {
    const [shapes, setShapes] = useState<Shape[]>([]);
    const [stops, setStops] = useState<Stop[]>([]);
    const [start, setStart] = useState<Stop>();
    const [end, setEnd] = useState<Stop>();
    const [date, setDate] = useState<Dayjs | null>(dayjs())
    const [journeys, setJourneys] = useState<EnrichedJourney[]>([]);

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
            .then(res => Promise.all(res.map(async j => await enrichJourney(j))))
            .then(journeys => setJourneys(journeys))
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
            <JourneyPolylines journey={journeys[0]} />

        </MapContainer>
    </div>
}