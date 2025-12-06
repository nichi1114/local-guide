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
      const images = action.payload.images;
      if (!state.localImages[placeId]) state.localImages[placeId] = [];
      state.localImages[placeId] = state.localImages[placeId].concat(images);
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

      state.localImages[placeId] = (state.localImages[placeId] || []).concat(images);
    },
    markImagesSaved: (state, action: PayloadAction<string>) => {
      const placeId = action.payload;
      if (!state.localImages[placeId]) return;

      state.localImages[placeId] = state.localImages[placeId].map((img) => ({
        ...img,
        saved: true,
      }));
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
      const { places, localImages } = action.payload;
      const normalizedLocalImages = Object.fromEntries(
        Object.entries(localImages).map(([placeId, imagesArray]) => [
          placeId,
          imagesArray.map((image) => ({
            ...image,
            saved: image.saved ?? true,
          })),
        ]),
      );
      state.places = places;
      state.localImages = normalizedLocalImages;
    });
    builder.addCase(savePlacesAsync.fulfilled, (state, action) => {
      const { places, localImages } = action.payload;
      state.places = places;
      state.localImages = localImages;
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
  markImagesSaved,
  clearLocalImages,
  markImageDeleted,
  clearDeletedImages,
} = placeSlice.actions;

export default placeSlice.reducer;
