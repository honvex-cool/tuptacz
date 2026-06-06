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