export type Place = {
  id: string;
  name: string;
  category: string;
  location: string;
  note: string;
};

export type LocalImage = {
  id: string; // image id
  uri: string; // image uri
  saved: boolean; // submitted or not
  caption?: string;
};
