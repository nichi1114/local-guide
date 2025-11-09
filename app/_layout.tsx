import { DarkTheme, DefaultTheme, ThemeProvider } from '@react-navigation/native';
import { Stack, usePathname, useRouter } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import 'react-native-reanimated';
import { ActivityIndicator, View } from 'react-native';
import { type ReactNode, useEffect } from 'react';
import { Provider } from 'react-redux';

import { useColorScheme } from '@/hooks/use-color-scheme';
import { store } from '@/store';
import { useAppDispatch, useAppSelector } from '@/store/hooks';
import {
  hydrateAuthSession,
} from '@/store/authSlice';
import { selectHasValidToken, selectIsHydratingAuth } from '@/store/authSelectors';

export const unstable_settings = {
  anchor: '(tabs)',
};

export default function RootLayout() {
  const colorScheme = useColorScheme();

  return (
    <Provider store={store}>
      <AuthGate>
        <ThemeProvider value={colorScheme === 'dark' ? DarkTheme : DefaultTheme}>
          <Stack>
            <Stack.Screen name="(tabs)" options={{ headerShown: false }} />
            <Stack.Screen name="modal" options={{ presentation: 'modal', title: 'Modal' }} />
            <Stack.Screen name="login" options={{ title: 'Sign In' }} />
          </Stack>
          <StatusBar style="auto" />
        </ThemeProvider>
      </AuthGate>
    </Provider>
  );
}

function AuthGate({ children }: { children: ReactNode }) {
  const dispatch = useAppDispatch();
  const isHydrating = useAppSelector(selectIsHydratingAuth);
  const hasValidToken = useAppSelector(selectHasValidToken);
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    dispatch(hydrateAuthSession());
  }, [dispatch]);

  useEffect(() => {
    if (isHydrating) {
      return;
    }

    if (!hasValidToken && pathname !== '/login') {
      router.replace('/login');
    } else if (hasValidToken && pathname === '/login') {
      router.replace('/');
    }
  }, [hasValidToken, isHydrating, pathname, router]);

  if (isHydrating) {
    return (
      <View style={{ flex: 1, alignItems: 'center', justifyContent: 'center' }}>
        <ActivityIndicator />
      </View>
    );
  }

  return children;
}
