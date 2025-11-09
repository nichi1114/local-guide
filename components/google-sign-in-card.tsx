import { ActivityIndicator, Pressable, StyleSheet, Text, View } from 'react-native';

import { useGoogleAuthSession } from '@/hooks/use-google-auth-session';
import { ThemedText } from '@/components/themed-text';
import { ThemedView } from '@/components/themed-view';

export function GoogleSignInCard() {
  const { signInWithGoogle, error, isLoading, session, ready, hasValidToken } =
    useGoogleAuthSession();

  const jwtHealthy = hasValidToken;

  return (
    <ThemedView style={styles.card}>
      <ThemedText type="subtitle">Sign in with Google</ThemedText>
      <ThemedText style={styles.description}>
        Use your Google account to finish the PKCE flow and let the backend mint a JWT.
      </ThemedText>
      <Pressable
        onPress={() => signInWithGoogle()}
        disabled={!ready || isLoading}
        style={({ pressed }) => [
          styles.button,
          (!ready || isLoading) && styles.buttonDisabled,
          pressed && styles.buttonPressed,
        ]}>
        {isLoading ? (
          <ActivityIndicator color="#fff" />
        ) : (
          <Text style={styles.buttonLabel}>Continue with Google</Text>
        )}
      </Pressable>
      {error ? (
        <Text style={styles.error} testID="google-auth-error">
          {error}
        </Text>
      ) : null}
      {session ? (
        <View style={styles.sessionBox}>
          <ThemedText type="defaultSemiBold">
            Signed in as {session.user.email ?? 'unknown'}
          </ThemedText>
          <Text style={styles.jwtLabel} numberOfLines={2}>
            JWT: {session.jwt_token}
          </Text>
          <Text style={[styles.jwtLabel, jwtHealthy ? styles.valid : styles.invalid]}>
            {jwtHealthy ? 'JWT is active' : 'JWT missing or expired'}
          </Text>
        </View>
      ) : null}
    </ThemedView>
  );
}

const styles = StyleSheet.create({
  card: {
    borderRadius: 12,
    padding: 16,
    gap: 12,
  },
  description: {
    lineHeight: 22,
  },
  button: {
    height: 48,
    borderRadius: 8,
    backgroundColor: '#1a73e8',
    alignItems: 'center',
    justifyContent: 'center',
  },
  buttonDisabled: {
    opacity: 0.5,
  },
  buttonPressed: {
    opacity: 0.85,
  },
  buttonLabel: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  error: {
    color: '#d93025',
  },
  sessionBox: {
    borderWidth: 1,
    borderColor: '#e1e3e1',
    borderRadius: 8,
    padding: 12,
    gap: 4,
  },
  jwtLabel: {
    fontSize: 12,
    color: '#666',
  },
  valid: {
    color: '#1a8a34',
  },
  invalid: {
    color: '#d93025',
  },
});
