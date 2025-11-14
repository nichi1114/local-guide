import Constants from 'expo-constants';
import { Platform } from 'react-native';

type ExtraConfig = {
  backendUrl?: string;
  googleIosClientId?: string;
  googleAndroidClientId?: string;
  googleClientId?: string;
  googleRedirectUri?: string;
  googleRedirectUriIos?: string;
  googleRedirectUriAndroid?: string;
  googleProviderId?: string;
  googleProviderIdIos?: string;
  googleProviderIdAndroid?: string;
};

const extra = (Constants.expoConfig?.extra ?? {}) as ExtraConfig;

const DEFAULT_BACKEND = 'http://localhost:8080';
const DEFAULT_PROVIDER = 'google';
const DEFAULT_PROVIDER_IOS = 'google-ios';
const DEFAULT_PROVIDER_ANDROID = 'google-android';

const envOr = (value: string | undefined, fallback?: string) =>
  value && value.length > 0 ? value : fallback;

const pickFirst = (...values: Array<string | undefined>) =>
  values.find((value) => value && value.length > 0);

export const API_BASE_URL =
  envOr(process.env.BACKEND_URL, extra.backendUrl) ?? DEFAULT_BACKEND;

const sharedClientId = envOr(process.env.GOOGLE_CLIENT_ID, extra.googleClientId);
const iosClientId =
  pickFirst(envOr(process.env.GOOGLE_IOS_CLIENT_ID, extra.googleIosClientId), sharedClientId) ??
  '';
const androidClientId =
  pickFirst(
    envOr(process.env.GOOGLE_ANDROID_CLIENT_ID, extra.googleAndroidClientId),
    sharedClientId,
  ) ?? '';
const webClientId =
  pickFirst(sharedClientId, iosClientId, androidClientId) ?? '';

const iosProviderId =
  envOr(process.env.GOOGLE_IOS_PROVIDER_NAME, extra.googleProviderIdIos) ??
  envOr(process.env.GOOGLE_PROVIDER_NAME, extra.googleProviderId) ??
  DEFAULT_PROVIDER_IOS;
const androidProviderId =
  envOr(process.env.GOOGLE_ANDROID_PROVIDER_NAME, extra.googleProviderIdAndroid) ??
  envOr(process.env.GOOGLE_PROVIDER_NAME, extra.googleProviderId) ??
  DEFAULT_PROVIDER_ANDROID;
const webProviderId =
  envOr(process.env.GOOGLE_PROVIDER_NAME, extra.googleProviderId) ??
  iosProviderId ??
  DEFAULT_PROVIDER;

export const GOOGLE_CLIENT_ID =
  Platform.OS === 'ios'
    ? iosClientId
    : Platform.OS === 'android'
      ? androidClientId
      : webClientId;

export const GOOGLE_PROVIDER_ID =
  Platform.OS === 'ios'
    ? iosProviderId
    : Platform.OS === 'android'
      ? androidProviderId
      : webProviderId;

const sharedRedirect = envOr(process.env.GOOGLE_REDIRECT_URI, extra.googleRedirectUri);
const redirectUriIos =
  pickFirst(
    envOr(process.env.GOOGLE_IOS_REDIRECT_URI, extra.googleRedirectUriIos),
    sharedRedirect,
  ) ?? '';
const redirectUriAndroid =
  pickFirst(
    envOr(process.env.GOOGLE_ANDROID_REDIRECT_URI, extra.googleRedirectUriAndroid),
    sharedRedirect,
  ) ?? '';
const redirectUriWeb =
  pickFirst(sharedRedirect, redirectUriIos, redirectUriAndroid) ?? '';

export const GOOGLE_REDIRECT_URI =
  Platform.OS === 'ios'
    ? redirectUriIos
    : Platform.OS === 'android'
      ? redirectUriAndroid
      : redirectUriWeb;
