import { LocalImage, Place } from "@/types/place";
import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { loadPlacesAsync, savePlacesAsync } from "./placeThunks";

interface PlaceState {
  userId: string | null;
  places: Place[];
  localImages: Record<string, LocalImage[]>; // placeId -> images
  deletedImages: Record<string, string[]>; // placeId -> deleted image ids
}

const initialState: PlaceState = {
  userId: null,
  places: [],
  localImages: {},
  deletedImages: {},
};

export const placeSlice = createSlice({
  name: "places",
  initialState,
  reducers: {
    setUserId: (state, action: PayloadAction<string>) => {
      state.userId = action.payload;
    },
    addPlace: (state, action: PayloadAction<{ place: Place; images: LocalImage[] }>) => {
      const newPlace = action.payload.place;
      state.places.push(newPlace);

      const placeId = newPlace.id;
      if (!state.localImages[placeId]) state.localImages[placeId] = [];
      state.localImages[placeId] = state.localImages[placeId].concat(action.payload.images);
    },
    updatePlace: (state, action: PayloadAction<{ id: string; updated: Omit<Place, "id"> }>) => {
      const { id, updated } = action.payload;
      state.places = state.places.map((p) => (p.id === id ? { ...p, ...updated } : p));
    },
    deletePlace: (state, action: PayloadAction<string>) => {
      state.places = state.places.filter((p) => p.id !== action.payload);
      delete state.localImages[action.payload];
      delete state.deletedImages[action.payload];
    },
    setPlaces: (state, action: PayloadAction<Place[]>) => {
      state.places = action.payload;
    },
    clearPlaces: (state) => {
      state.places = [];
      state.localImages = {};
      state.deletedImages = {};
    },
    addLocalImages: (state, action: PayloadAction<{ placeId: string; images: LocalImage[] }>) => {
      const { placeId, images } = action.payload;
      if (!state.localImages[placeId]) state.localImages[placeId] = [];
      state.localImages[placeId] = state.localImages[placeId].concat(images);
    },
    clearLocalImages: (state, action: PayloadAction<string>) => {
      delete state.localImages[action.payload];
    },
    markImageDeleted: (state, action: PayloadAction<{ placeId: string; imageIds: string[] }>) => {
      const { placeId, imageIds } = action.payload;
      if (!state.deletedImages[placeId]) state.deletedImages[placeId] = [];
      imageIds.forEach((id) => {
        if (!state.deletedImages[placeId].includes(id)) state.deletedImages[placeId].push(id);
        state.localImages[placeId] = (state.localImages[placeId] || []).filter(
          (img) => img.id !== id,
        );
      });
    },
    clearDeletedImages: (state, action: PayloadAction<string>) => {
      delete state.deletedImages[action.payload];
    },
  },
  extraReducers: (builder) => {
    builder.addCase(loadPlacesAsync.fulfilled, (state, action) => {
      state.places = action.payload.places || [];
      state.localImages = action.payload.localImages || {};
    });
    builder.addCase(savePlacesAsync.fulfilled, (state, action) => {
      state.places = action.payload.places;
      state.localImages = action.payload.localImages;
    });
  },
});

export const {
  setUserId,
  addPlace,
  updatePlace,
  deletePlace,
  setPlaces,
  clearPlaces,
  addLocalImages,
  clearLocalImages,
  markImageDeleted,
  clearDeletedImages,
} = placeSlice.actions;

export default placeSlice.reducer;
