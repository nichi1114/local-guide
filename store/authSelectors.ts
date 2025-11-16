import { createSelector } from "@reduxjs/toolkit";

import { isJwtValid } from "@/utils/jwt";
import type { RootState } from "./index";

export const selectAuthSession = (state: RootState) => state.auth.session;
export const selectIsHydratingAuth = (state: RootState) => state.auth.isHydrating;

export const selectHasValidToken = createSelector(selectAuthSession, (session) =>
  isJwtValid(session?.jwt_token),
);
