export type RouteType = "Bus" | "Tram";

export type LatLng = {
  latitude: number;
  longitude: number;
};

export type Shape = {
  id: string;
  points: LatLng[];
};

export type Stop = {
  id: string;
  code?: string;
  name: string;
  position: LatLng;
};

export type Route = {
  short_name: string
}

export type StopTime = {
  stop_id: number,
  arrival_time: number,
  departure_time: number
}

export type Trip = {
  route: Route,
  stop_times: StopTime[],
  stops: Stop[],
}

export type JourneyStop = {
  stop_id: number,
  stop_name: string,
  arrival_time: number,
  position: LatLng
}

export type JourneyLeg = {
  trip_id: number
  route_name: string
  route_type: RouteType
  stops: JourneyStop[],
  walked_distance: number
}

export type Journey = {
  legs: JourneyLeg[]
}

export function legDepartureTime(leg: JourneyLeg) {
  return leg.stops.at(0)!.arrival_time;
}
export function legArrivalTime(leg: JourneyLeg) {
  return leg.stops.at(-1)!.arrival_time;
}

export function journeyDepartureTime(journey: Journey) {
  return legDepartureTime(journey.legs.at(0)!)
}
export function journeyArrivalTime(journey: Journey) {
  return legArrivalTime(journey.legs.at(-1)!)
}
