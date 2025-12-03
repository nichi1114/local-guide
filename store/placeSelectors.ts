import { createSelector } from "@reduxjs/toolkit";
import { RootState } from ".";

export const selectPlaceUserId = (state: RootState) => state.poi.userId;
export const selectPlaces = (state: RootState) => state.poi.places;
export const selectPlaceById = (state: RootState, id: string) =>
  state.poi.places.find((item) => item.id === id);

export const selectLocalImages = (state: RootState) => state.poi.localImages;

export const makeSelectImagesByPlaceId = (placeId: string) =>
  createSelector(
    (state: RootState) => state.poi.localImages,
    (localImages) => localImages[placeId] || [],
  );
