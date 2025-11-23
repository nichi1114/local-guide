import { StyleSheet } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { GoogleSignInCard } from "@/components/google-sign-in-card";

export default function LoginScreen() {
  return (
    <SafeAreaView style={styles.container}>
      <GoogleSignInCard />
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 24,
    justifyContent: "center",
  },
});
