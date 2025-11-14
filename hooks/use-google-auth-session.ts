import { useCallback, useMemo, useState } from "react";
import { Platform } from "react-native";
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

const discovery: AuthSession.DiscoveryDocument = {
  authorizationEndpoint: "https://accounts.google.com/o/oauth2/v2/auth",
  tokenEndpoint: "https://oauth2.googleapis.com/token",
};

const redirectUri =
  GOOGLE_REDIRECT_URI ||
  AuthSession.makeRedirectUri({
    scheme: "com.ece1778.localguide",
    preferLocalhost: true,
  });

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
    discovery,
  );

  const ready = useMemo(() => Boolean(request), [request]);

  const signInWithGoogle = useCallback(async () => {
    if (!GOOGLE_CLIENT_ID) {
      setError(
        "Missing Google client ID. Provide GOOGLE_CLIENT_ID/GOOGLE_IOS_CLIENT_ID/GOOGLE_ANDROID_CLIENT_ID or expo.extra Google client values.",
      );
      return null;
    }

    if (!request) {
      setError("Google auth request is still preparing. Please try again.");
      return null;
    }

    setIsLoading(true);
    setError(null);

    try {
      const result = await promptAsync({
        useProxy: Platform.OS !== "web",
      });

      if (result.type !== "success") {
        if (result.type !== "cancel" && result.type !== "dismiss") {
          setError(result.error ?? "Google sign-in did not complete.");
        }
        return null;
      }

      const code = result.params?.code;
      if (!code) {
        setError("Google response did not include an authorization code.");
        return null;
      }

      if (!request.codeVerifier) {
        setError("PKCE code verifier is missing. Restart the sign-in flow.");
        return null;
      }

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

      if (!response.ok) {
        const errorPayload = await safeParseError(response);
        const detail = errorPayload?.message ?? `Backend responded with ${response.status}.`;
        throw new Error(detail);
      }

      const payload = (await response.json()) as BackendLoginResponse;
      await dispatch(persistAuthSession(payload)).unwrap();
      return payload;
    } catch (authError) {
      if (authError instanceof DOMException && authError.name === "AbortError") {
        setError("Sign-in timed out. Check your connection and try again.");
      } else {
        const message =
          authError instanceof Error ? authError.message : "Unexpected error while signing in.";
        setError(message);
      }
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
