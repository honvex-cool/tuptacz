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

export type JourneyStep = {
    trip_id: number,
    start_stop_idx: number,
    end_stop_idx: number,
    walked_distance: number
}

export type Journey = {
    arrival_time: number,
    steps: JourneyStep[]
}