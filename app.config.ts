import { ConfigContext, ExpoConfig } from 'expo/config';
import 'dotenv/config';

const APP_NAME = 'local-guide';
const APP_IDENTIFIER = 'com.ece1778.localguide';
const DEFAULT_BACKEND_URL = 'http://localhost:8080';
const DEFAULT_GOOGLE_PROVIDER = 'google';
const DEFAULT_GOOGLE_PROVIDER_IOS = 'google-ios';
const DEFAULT_GOOGLE_PROVIDER_ANDROID = 'google-android';

export default ({ config }: ConfigContext): ExpoConfig => ({
  ...config,
  name: APP_NAME,
  slug: APP_NAME,
  version: '1.0.0',
  orientation: 'portrait',
  icon: './assets/images/icon.png',
  scheme: APP_IDENTIFIER,
  userInterfaceStyle: 'automatic',
  newArchEnabled: true,
  ios: {
    ...(config.ios ?? {}),
    supportsTablet: true,
    bundleIdentifier: process.env.IOS_BUNDLE_IDENTIFIER ?? APP_IDENTIFIER,
  },
  android: {
    ...(config.android ?? {}),
    adaptiveIcon: {
      backgroundColor: '#E6F4FE',
      foregroundImage: './assets/images/android-icon-foreground.png',
      backgroundImage: './assets/images/android-icon-background.png',
      monochromeImage: './assets/images/android-icon-monochrome.png',
    },
    edgeToEdgeEnabled: true,
    predictiveBackGestureEnabled: false,
    package: process.env.ANDROID_PACKAGE ?? APP_IDENTIFIER,
  },
  web: {
    ...(config.web ?? {}),
    output: 'static',
    favicon: './assets/images/favicon.png',
  },
  plugins: [
    'expo-router',
    [
      'expo-splash-screen',
      {
        image: './assets/images/splash-icon.png',
        imageWidth: 200,
        resizeMode: 'contain',
        backgroundColor: '#ffffff',
        dark: {
          backgroundColor: '#000000',
        },
      },
    ],
  ],
  experiments: {
    ...(config.experiments ?? {}),
    typedRoutes: true,
    reactCompiler: true,
  },
  extra: {
    ...(config.extra ?? {}),
    backendUrl: process.env.BACKEND_URL ?? DEFAULT_BACKEND_URL,
    googleClientId: process.env.GOOGLE_CLIENT_ID ?? '',
    googleIosClientId: process.env.GOOGLE_IOS_CLIENT_ID ?? '',
    googleAndroidClientId: process.env.GOOGLE_ANDROID_CLIENT_ID ?? '',
    googleRedirectUri: process.env.GOOGLE_REDIRECT_URI ?? '',
    googleRedirectUriIos: process.env.GOOGLE_IOS_REDIRECT_URI ?? '',
    googleRedirectUriAndroid: process.env.GOOGLE_ANDROID_REDIRECT_URI ?? '',
    googleProviderId: process.env.GOOGLE_PROVIDER_NAME ?? DEFAULT_GOOGLE_PROVIDER,
    googleProviderIdIos:
      process.env.GOOGLE_IOS_PROVIDER_NAME ?? DEFAULT_GOOGLE_PROVIDER_IOS,
    googleProviderIdAndroid:
      process.env.GOOGLE_ANDROID_PROVIDER_NAME ?? DEFAULT_GOOGLE_PROVIDER_ANDROID,
  },
});
