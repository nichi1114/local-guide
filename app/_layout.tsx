import { DarkTheme, DefaultTheme, ThemeProvider } from '@react-navigation/native';
import { Stack, usePathname, useRouter } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import 'react-native-reanimated';
import { ActivityIndicator, View } from 'react-native';
import { type ReactNode, useEffect } from 'react';

import { useColorScheme } from '@/hooks/use-color-scheme';
import { AuthProvider, useAuth } from '@/contexts/auth-context';

export const unstable_settings = {
  anchor: '(tabs)',
};

export default function RootLayout() {
  const colorScheme = useColorScheme();

  return (
    <AuthProvider>
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
    </AuthProvider>
  );
}

function AuthGate({ children }: { children: ReactNode }) {
  const { isHydrating, hasValidToken } = useAuth();
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    if (isHydrating) {
      return;
    }

    const tokenValid = hasValidToken();
    if (!tokenValid && pathname !== '/login') {
      router.replace('/login');
    } else if (tokenValid && pathname === '/login') {
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
