import { DarkTheme, DefaultTheme, ThemeProvider } from "@react-navigation/native";
import { Stack, usePathname, useRouter } from "expo-router";
import { StatusBar } from "expo-status-bar";
import { type ReactNode, useEffect, useRef } from "react";
import { ActivityIndicator, View } from "react-native";
import "react-native-reanimated";
import { Provider } from "react-redux";

import { useColorScheme } from "@/hooks/use-color-scheme";
import { store } from "@/store";
import { selectHasValidToken, selectIsHydratingAuth } from "@/store/authSelectors";
import { hydrateAuthSession } from "@/store/authSlice";
import { useAppDispatch, useAppSelector } from "@/store/hooks";

export const unstable_settings = {
  anchor: "(tabs)",
};

export default function RootLayout() {
  const colorScheme = useColorScheme();

  return (
    <Provider store={store}>
      <AuthGate>
        <ThemeProvider value={colorScheme === "dark" ? DarkTheme : DefaultTheme}>
          <Stack>
            <Stack.Screen name="login" options={{ title: "Sign In" }} />
            <Stack.Screen name="index" options={{ title: "Home" }} />
            <Stack.Screen name="add-edit" options={{ title: "" }} />
            <Stack.Screen name="places/[id]" options={{ title: "Place of Interest Details" }} />
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
  const pendingRouteRef = useRef<string | null>(null);

  useEffect(() => {
    dispatch(hydrateAuthSession());
  }, [dispatch]);

  useEffect(() => {
    if (pendingRouteRef.current && pathname === pendingRouteRef.current) {
      pendingRouteRef.current = null;
    }
  }, [pathname]);

  useEffect(() => {
    if (isHydrating) {
      return;
    }

    const needsLogin = !hasValidToken;
    const isOnLogin = pathname === "/login";

    let targetRoute: string | null = null;
    if (needsLogin && !isOnLogin) {
      targetRoute = "/login";
    } else if (!needsLogin && isOnLogin) {
      targetRoute = "/";
    }

    if (!targetRoute) {
      return;
    }

    if (pendingRouteRef.current && pendingRouteRef.current === targetRoute) {
      return;
    }

    pendingRouteRef.current = targetRoute;
    router.replace(targetRoute);
    return () => {
      pendingRouteRef.current = null;
    };
  }, [hasValidToken, isHydrating, pathname, router]);

  if (isHydrating) {
    return (
      <View
        style={{ flex: 1, alignItems: "center", justifyContent: "center" }}
        accessibilityRole="progressbar"
      >
        <ActivityIndicator accessibilityLabel="Loading authentication status" />
      </View>
    );
  }

  return children;
}
