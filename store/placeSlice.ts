import { Place } from "@/types/place";
import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { RootState } from ".";
import { loadPlacesAsync, savePlacesAsync } from "./placeThunks";

interface PlaceState {
  userId: string | null;
  places: Place[];
}

const initialState: PlaceState = {
  userId: null,
  places: [],
};

export const placeSlice = createSlice({
  name: "places",
  initialState,
  reducers: {
    setUserId: (state, action: PayloadAction<string>) => {
      state.userId = action.payload;
    },
    addPlace: (state, action: PayloadAction<Omit<Place, "id">>) => {
      const newOne: Place = {
        id: Date.now().toString(),
        ...action.payload,
      };
      const newPlaces = [...state.places, newOne];
      state.places = newPlaces;
    },
    updatePlace: (state, action: PayloadAction<{ id: string; updated: Omit<Place, "id"> }>) => {
      const id = action.payload.id;
      const newPlace = action.payload.updated;
      const newPlaces = state.places.map((item) =>
        item.id === id
          ? {
              ...item,
              name: newPlace.name,
              category: newPlace.category,
              location: newPlace.location,
              note: newPlace.note,
            }
          : item,
      );
      state.places = newPlaces;
    },
    deletePlace: (state, action: PayloadAction<string>) => {
      const newPlaces = state.places.filter((a) => a.id !== action.payload);
      state.places = newPlaces;
    },
    setPlaces: (state, action: PayloadAction<Place[]>) => {
      state.places = action.payload;
    },
    clearPlaces: (state) => {
      state.places = [];
    },
  },
  extraReducers: (builder) => {
    builder.addCase(loadPlacesAsync.fulfilled, (state, action) => {
      state.places = action.payload;
    });
    builder.addCase(savePlacesAsync.fulfilled, (state, action) => {
      state.places = action.payload;
    });
  },
});
export const { setUserId, addPlace, updatePlace, deletePlace, setPlaces } = placeSlice.actions;

export const selectUserId = (state: RootState) => state.poi.userId;
export const selectPlaces = (state: RootState) => state.poi.places;
export const selectPlaceById = (state: RootState, id: string) =>
  state.poi.places.find((item) => item.id === id);

export default placeSlice.reducer;
