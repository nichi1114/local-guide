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
  filename?: string; // for backend sync
  caption?: string;
};
