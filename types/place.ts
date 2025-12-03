export type Place = {
  id: string;
  name: string;
  category: string;
  location: string;
  note: string;
};

export type LocalImage = {
  id: string;
  uri: string;
  saved: boolean;
  caption?: string;
};
