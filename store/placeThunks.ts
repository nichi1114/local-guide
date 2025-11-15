import { Place } from "@/types/place";
import AsyncStorage from "@react-native-async-storage/async-storage";
import { createAsyncThunk } from "@reduxjs/toolkit";

const placesKeyPrefix = "@LocalGuide_places";

export const loadPlacesAsync = createAsyncThunk(
  "places/loadPlacesForUser",
  async (userId: string) => {
    try {
      const placesKey = `${placesKeyPrefix}${userId}`;
      const saved = await AsyncStorage.getItem(placesKey);
      return saved ? JSON.parse(saved) : [];
    } catch {
      return [];
    }
  },
);

export const savePlacesAsync = createAsyncThunk<Place[], { userId: string; places: Place[] }>(
  "places/savePlacesForUser",
  async ({ userId, places }) => {
    try {
      const placesKey = `${placesKeyPrefix}${userId}`;
      await AsyncStorage.setItem(placesKey, JSON.stringify(places));
      return places;
    } catch {
      return [];
    }
  },
);
