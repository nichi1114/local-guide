import { ConfigContext, ExpoConfig } from 'expo/config';
import 'dotenv/config';

const APP_NAME = 'local-guide';
const DEFAULT_BACKEND_URL = 'http://localhost:8080';

export default ({ config }: ConfigContext): ExpoConfig => ({
  ...config,
  name: APP_NAME,
  slug: APP_NAME,
  version: '1.0.0',
  orientation: 'portrait',
  icon: './assets/images/icon.png',
  scheme: 'localguide',
  userInterfaceStyle: 'automatic',
  newArchEnabled: true,
  ios: {
    ...(config.ios ?? {}),
    supportsTablet: true,
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
    googleRedirectUri: process.env.GOOGLE_REDIRECT_URI ?? '',
  },
});
