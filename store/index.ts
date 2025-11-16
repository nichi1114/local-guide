import { configureStore } from "@reduxjs/toolkit";

import authReducer from "./authSlice";
import poiReducer from "./placeSlice";

export const store = configureStore({
  reducer: {
    auth: authReducer,
    poi: poiReducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
