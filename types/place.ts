export type Place = {
  id: string;
  name: string;
  category: string;
  location: string;
  note: string;
  // todo images: PlaceImage[];
};

export type PlaceImage = {
  id: string;
  filename: string;
  caption: string | null;
};
