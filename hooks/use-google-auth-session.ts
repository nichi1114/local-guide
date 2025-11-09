import { useCallback, useMemo, useState } from 'react';
import { Platform } from 'react-native';
import * as AuthSession from 'expo-auth-session';
import * as WebBrowser from 'expo-web-browser';

import { API_BASE_URL, GOOGLE_CLIENT_ID } from '@/constants/env';
import { useAuth } from '@/contexts/auth-context';
import type { BackendLoginResponse } from '@/types/auth';

WebBrowser.maybeCompleteAuthSession();

const discovery: AuthSession.DiscoveryDocument = {
  authorizationEndpoint: 'https://accounts.google.com/o/oauth2/v2/auth',
  tokenEndpoint: 'https://oauth2.googleapis.com/token',
};

const redirectUri = AuthSession.makeRedirectUri({
  scheme: 'localguide',
  preferLocalhost: true,
});

export function useGoogleAuthSession() {
  const { saveSession, session, hasValidToken } = useAuth();
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const [request, , promptAsync] = AuthSession.useAuthRequest(
    {
      clientId: GOOGLE_CLIENT_ID,
      redirectUri,
      responseType: AuthSession.ResponseType.Code,
      scopes: ['openid', 'profile', 'email'],
      codeChallengeMethod: AuthSession.CodeChallengeMethod.S256,
      extraParams: {
        access_type: 'offline',
        prompt: 'consent',
      },
    },
    discovery
  );

  const ready = useMemo(() => Boolean(request), [request]);

  const signInWithGoogle = useCallback(async () => {
    if (!GOOGLE_CLIENT_ID) {
      setError(
        'Missing Google client ID. Provide EXPO_PUBLIC_GOOGLE_CLIENT_ID or expo.extra.googleClientId.'
      );
      return null;
    }

    if (!request) {
      setError('Google auth request is still preparing. Please try again.');
      return null;
    }

    setIsLoading(true);
    setError(null);

    try {
      const result = await promptAsync({
        useProxy: Platform.OS !== 'web',
      });

      if (result.type !== 'success') {
        if (result.type !== 'cancel' && result.type !== 'dismiss') {
          setError(result.error ?? 'Google sign-in did not complete.');
        }
        return null;
      }

      const code = result.params?.code;
      if (!code) {
        setError('Google response did not include an authorization code.');
        return null;
      }

      if (!request.codeVerifier) {
        setError('PKCE code verifier is missing. Restart the sign-in flow.');
        return null;
      }

      const response = await fetch(`${API_BASE_URL}/auth/google/callback`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          code,
          code_verifier: request.codeVerifier,
        }),
      });

      if (!response.ok) {
        const errorPayload = await safeParseError(response);
        const detail = errorPayload?.message ?? `Backend responded with ${response.status}.`;
        throw new Error(detail);
      }

      const payload = (await response.json()) as BackendLoginResponse;
      await saveSession(payload);
      return payload;
    } catch (authError) {
      const message =
        authError instanceof Error ? authError.message : 'Unexpected error while signing in.';
      setError(message);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, [promptAsync, request, saveSession]);

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
