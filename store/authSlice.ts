import * as SecureStore from "expo-secure-store";
import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";

import type { BackendLoginResponse } from "@/types/auth";
import { isJwtValid } from "@/utils/jwt";

const STORAGE_KEY = "LOCAL_GUIDE_AUTH_SESSION";

export const hydrateAuthSession = createAsyncThunk("auth/hydrate", async () => {
  const raw = await SecureStore.getItemAsync(STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as BackendLoginResponse;
    if (!isJwtValid(parsed.jwt_token)) {
      await SecureStore.deleteItemAsync(STORAGE_KEY);
      return null;
    }
    return parsed;
  } catch (error) {
    console.warn("Failed to parse persisted session", error);
    await SecureStore.deleteItemAsync(STORAGE_KEY);
    return null;
  }
});

export const persistAuthSession = createAsyncThunk(
  "auth/persist",
  async (session: BackendLoginResponse) => {
    try {
      await SecureStore.setItemAsync(STORAGE_KEY, JSON.stringify(session));
      return session;
    } catch (error) {
      console.error("Failed to persist auth session", error);
      throw error;
    }
  },
);

export const clearAuthSession = createAsyncThunk("auth/clear", async () => {
  await SecureStore.deleteItemAsync(STORAGE_KEY);
  return null;
});

type AuthState = {
  session: BackendLoginResponse | null;
  isHydrating: boolean;
};

const initialState: AuthState = {
  session: null,
  isHydrating: true,
};

const authSlice = createSlice({
  name: "auth",
  initialState,
  reducers: {},
  extraReducers: (builder) => {
    builder
      .addCase(hydrateAuthSession.pending, (state) => {
        state.isHydrating = true;
      })
      .addCase(hydrateAuthSession.fulfilled, (state, action) => {
        state.session = action.payload ?? null;
        state.isHydrating = false;
      })
      .addCase(hydrateAuthSession.rejected, (state) => {
        state.session = null;
        state.isHydrating = false;
      })
      .addCase(persistAuthSession.fulfilled, (state, action) => {
        state.session = action.payload;
      })
      .addCase(persistAuthSession.rejected, (state) => {
        state.session = null;
      })
      .addCase(clearAuthSession.fulfilled, (state) => {
        state.session = null;
      });
  },
});

export default authSlice.reducer;
