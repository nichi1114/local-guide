import { Place } from "@/types/place";
import AsyncStorage from "@react-native-async-storage/async-storage";
import { createAsyncThunk } from "@reduxjs/toolkit";
import { RootState } from ".";

const placesKeyPrefix = "@LocalGuide_places";

export const loadPlacesAsync = createAsyncThunk(
  "places/loadPlacesForUser",
  async (userId: string) => {
    try {
      const placesKey = `${placesKeyPrefix}${userId}`;
      const saved = await AsyncStorage.getItem(placesKey);
      return saved ? JSON.parse(saved) : [];
    } catch (err) {
      console.error(`Failed to load places for ${userId}: `, err);
      return [];
    }
  },
);

export const savePlacesAsync = createAsyncThunk<Place[], string, { state: RootState }>(
  "places/savePlacesForUser",
  async (userId, { getState }) => {
    try {
      const places = (getState() as RootState).poi.places;
      const placesKey = `${placesKeyPrefix}${userId}`;
      await AsyncStorage.setItem(placesKey, JSON.stringify(places));
      return places;
    } catch (err) {
      console.error(`Failed to save places for ${userId}: `, err);
      return [];
    }
  },
);
