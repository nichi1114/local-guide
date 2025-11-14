import { createSelector } from '@reduxjs/toolkit';

import type { RootState } from './index';
import { isJwtValid } from '@/utils/jwt';

export const selectAuthSession = (state: RootState) => state.auth.session;
export const selectIsHydratingAuth = (state: RootState) => state.auth.isHydrating;

export const selectHasValidToken = createSelector(selectAuthSession, (session) =>
  isJwtValid(session?.jwt_token)
);
