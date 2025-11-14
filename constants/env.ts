import Constants from 'expo-constants';

type ExtraConfig = {
  backendUrl?: string;
  googleClientId?: string;
};

const extra = (Constants.expoConfig?.extra ?? {}) as ExtraConfig;

export const API_BASE_URL =
  process.env.BACKEND_URL ?? extra.backendUrl ?? 'http://localhost:8080';

export const GOOGLE_CLIENT_ID = process.env.GOOGLE_CLIENT_ID ?? extra.googleClientId ?? '';
