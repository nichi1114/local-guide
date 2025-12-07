import { useCallback, useMemo, useState } from "react";
import { Alert, Platform } from "react-native";
import * as AuthSession from "expo-auth-session";
import * as WebBrowser from "expo-web-browser";

import {
  API_BASE_URL,
  GOOGLE_CLIENT_ID,
  GOOGLE_PROVIDER_ID,
  GOOGLE_REDIRECT_URI,
} from "@/constants/env";
import type { BackendLoginResponse } from "@/types/auth";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { selectAuthSession, selectHasValidToken } from "@/store/authSelectors";
import { persistAuthSession } from "@/store/authSlice";

WebBrowser.maybeCompleteAuthSession();

const GOOGLE_DISCOVERY: AuthSession.DiscoveryDocument = {
  authorizationEndpoint: "https://accounts.google.com/o/oauth2/v2/auth",
  tokenEndpoint: "https://oauth2.googleapis.com/token",
} as const;

const redirectUri =
  GOOGLE_REDIRECT_URI ||
  AuthSession.makeRedirectUri({
    scheme: "com.ece1778.localguide",
    preferLocalhost: true,
  });
const shouldUseProxy = GOOGLE_REDIRECT_URI ? false : Platform.OS !== "web";

export function useGoogleAuthSession() {
  const dispatch = useAppDispatch();
  const session = useAppSelector(selectAuthSession);
  const hasValidToken = useAppSelector(selectHasValidToken);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const [request, , promptAsync] = AuthSession.useAuthRequest(
    {
      clientId: GOOGLE_CLIENT_ID,
      redirectUri,
      responseType: AuthSession.ResponseType.Code,
      scopes: ["openid", "profile", "email"],
      codeChallengeMethod: AuthSession.CodeChallengeMethod.S256,
      extraParams: {
        access_type: "offline",
        prompt: "consent",
      },
    },
    GOOGLE_DISCOVERY,
  );

  const ready = useMemo(() => Boolean(request), [request]);

  const showAuthDebugAlert = useCallback(() => {
    const payload = {
      platform: Platform.OS,
      providerId: GOOGLE_PROVIDER_ID,
      redirectUri,
      useProxy: shouldUseProxy,
      requestReady: Boolean(request),
      hasCodeVerifier: Boolean(request?.codeVerifier),
      hasClientId: Boolean(GOOGLE_CLIENT_ID),
      hasSession: Boolean(session),
      hasJwtToken: Boolean(session?.jwt_token),
    };
    Alert.alert("Google Auth Debug", JSON.stringify(payload, null, 2));
  }, [request, session]);

  const signInWithGoogle = useCallback(async () => {
    const debugAlert = (message: string, details?: Record<string, unknown>) => {
      const suffix = details ? `\n${JSON.stringify(details, null, 2)}` : "";
      Alert.alert("Google Auth Debug", `${message}${suffix}`);
    };

    if (!GOOGLE_CLIENT_ID) {
      debugAlert("Missing Google client ID configuration");
      setError(
        "Missing Google client ID configuration. Please set the appropriate environment variable for your platform.",
      );
      return null;
    }

    if (!request) {
      debugAlert("Auth request is still preparing");
      setError("Google auth request is still preparing. Please try again.");
      return null;
    }

    debugAlert("Starting Google sign-in", {
      redirectUri,
      platform: Platform.OS,
      providerId: GOOGLE_PROVIDER_ID,
    });
    setIsLoading(true);
    setError(null);

    try {
      const result = await promptAsync({
        useProxy: shouldUseProxy,
      });

      debugAlert("Auth session completed", {
        type: result.type,
        hasCode: Boolean(result.params?.code),
      });

      if (result.type !== "success") {
        if (result.type !== "cancel" && result.type !== "dismiss") {
          setError(result.error ?? "Google sign-in did not complete.");
        }
        return null;
      }

      const code = result.params?.code;
      if (!code) {
        debugAlert("Missing authorization code from Google response");
        setError("Google response did not include an authorization code.");
        return null;
      }

      if (!request.codeVerifier) {
        debugAlert("Missing PKCE code verifier");
        setError("PKCE code verifier is missing. Restart the sign-in flow.");
        return null;
      }

      debugAlert("Sending auth code to backend", {
        endpoint: `${API_BASE_URL}/auth/${GOOGLE_PROVIDER_ID}/callback`,
      });
      const response = await fetchWithTimeout(
        `${API_BASE_URL}/auth/${GOOGLE_PROVIDER_ID}/callback`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            code,
            code_verifier: request.codeVerifier,
          }),
        },
        10000,
      );

      debugAlert("Backend responded", { status: response.status });

      if (!response.ok) {
        const errorPayload = await safeParseError(response);
        const detail = errorPayload?.message ?? `Backend responded with ${response.status}.`;
        throw new Error(detail);
      }

      const payload = (await response.json()) as BackendLoginResponse;
      await dispatch(persistAuthSession(payload)).unwrap();
      debugAlert("Login successful; session persisted");
      return payload;
    } catch (authError) {
      const isAbortError =
        authError instanceof Error && authError.name === "AbortError";
      if (isAbortError) {
        setError("Sign-in timed out. Check your connection and try again.");
      } else {
        const message =
          authError instanceof Error ? authError.message : "Unexpected error while signing in.";
        setError(message);
      }
      debugAlert("Login failed", {
        error:
          authError instanceof Error ? authError.message : "unknown error",
      });
      return null;
    } finally {
      setIsLoading(false);
    }
  }, [dispatch, promptAsync, request]);

  return {
    signInWithGoogle,
    session,
    isLoading,
    error,
    ready,
    hasValidToken,
    showAuthDebugAlert,
  };
}

async function safeParseError(response: Response) {
  try {
    return (await response.json()) as { message?: string };
  } catch {
    return null;
  }
}

async function fetchWithTimeout(
  resource: RequestInfo | URL,
  options?: RequestInit,
  timeoutMs = 10000,
) {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetch(resource, {
      ...options,
      signal: controller.signal,
    });
    return response;
  } finally {
    clearTimeout(timeoutId);
  }
}
