import { LocalImage, Place } from "@/types/place";
import AsyncStorage from "@react-native-async-storage/async-storage";
import { createAsyncThunk } from "@reduxjs/toolkit";
import { RootState } from ".";

const placesKeyPrefix = "@LocalGuide_places";
const localImagesKeyPrefix = "@LocalGuide_localImages";

// Load places & localImages from AsyncStorage
export const loadPlacesAsync = createAsyncThunk(
  "places/loadPlacesForUser",
  async (userId: string) => {
    try {
      const placesKey = `${placesKeyPrefix}${userId}`;
      const savedPlaces = await AsyncStorage.getItem(placesKey);
      const places: Place[] = savedPlaces ? JSON.parse(savedPlaces) : [];

      const imagesKey = `${localImagesKeyPrefix}${userId}`;
      const savedImages = await AsyncStorage.getItem(imagesKey);
      const localImages: Record<string, LocalImage[]> = savedImages ? JSON.parse(savedImages) : {};

      return { places, localImages };
    } catch (err) {
      console.error(err);
      return { places: [], localImages: {} };
    }
  },
);

// Save places & localImages to AsyncStorage
export const savePlacesAsync = createAsyncThunk<
  { places: Place[]; localImages: Record<string, LocalImage[]> },
  string,
  { state: RootState }
>("places/savePlacesForUser", async (userId, { getState }) => {
  try {
    const state = getState().poi;

    await AsyncStorage.setItem(`${placesKeyPrefix}${userId}`, JSON.stringify(state.places));
    await AsyncStorage.setItem(
      `${localImagesKeyPrefix}${userId}`,
      JSON.stringify(state.localImages),
    );

    return { places: state.places, localImages: state.localImages };
  } catch (err) {
    console.error("Failed to save places and images for userId:", userId, err);
    throw err;
  }
});
