import Constants from "expo-constants";

type ExtraConfig = {
  apiBaseUrl?: string;
  googleClientId?: string;
};

const extra = (Constants.expoConfig?.extra ?? {}) as ExtraConfig;

export const API_BASE_URL =
  process.env.LOCAL_GUIDE_API_BASE_URL ?? extra.apiBaseUrl ?? "http://localhost:8080";

export const GOOGLE_CLIENT_ID = process.env.LOCAL_GUIDE_CLIENT_ID ?? extra.googleClientId ?? "";
