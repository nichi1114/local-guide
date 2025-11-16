import { ActivityIndicator, Pressable, StyleSheet, Text, View } from "react-native";

import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { useGoogleAuthSession } from "@/hooks/use-google-auth-session";

export function GoogleSignInCard() {
  const { signInWithGoogle, error, isLoading, session, ready, hasValidToken } =
    useGoogleAuthSession();

  const sessionExpired = Boolean(session && !hasValidToken);
  const hasActiveSession = Boolean(session && hasValidToken);
  const errorMessage = sessionExpired
    ? "Session expired. Please sign in again."
    : error
      ? "Sign-in failed. Please try again."
      : null;

  return (
    <ThemedView style={styles.card}>
      <ThemedText type="subtitle">Sign in with Google</ThemedText>
      <ThemedText style={styles.description}>
        Use your Google account to sign in securely and continue using the app.
      </ThemedText>
      <Pressable
        accessibilityRole="button"
        accessibilityLabel="Sign in with Google"
        accessibilityState={{ disabled: !ready || isLoading }}
        onPress={() => signInWithGoogle()}
        disabled={!ready || isLoading}
        style={({ pressed }) => [
          styles.button,
          (!ready || isLoading) && styles.buttonDisabled,
          pressed && styles.buttonPressed,
        ]}
      >
        {isLoading ? (
          <ActivityIndicator color="#fff" accessibilityLabel="Signing in with Google" />
        ) : (
          <Text style={styles.buttonLabel}>Continue with Google</Text>
        )}
      </Pressable>
      {errorMessage ? (
        <Text
          style={styles.error}
          testID="google-auth-error"
          accessibilityRole="alert"
          accessibilityLiveRegion="polite"
        >
          {errorMessage}
        </Text>
      ) : null}
      {hasActiveSession ? (
        <View style={styles.sessionBox}>
          <ThemedText type="defaultSemiBold">
            Signed in as {session?.user.email ?? "unknown"}
          </ThemedText>
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
    backgroundColor: "#1a73e8",
    alignItems: "center",
    justifyContent: "center",
  },
  buttonDisabled: {
    opacity: 0.5,
  },
  buttonPressed: {
    opacity: 0.85,
  },
  buttonLabel: {
    color: "#fff",
    fontSize: 16,
    fontWeight: "600",
  },
  error: {
    color: "#d93025",
  },
  sessionBox: {
    borderWidth: 1,
    borderColor: "#e1e3e1",
    borderRadius: 8,
    padding: 12,
    gap: 4,
  },
});
