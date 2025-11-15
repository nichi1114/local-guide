// Reuse the Card component implementation from Assignment.
import { StyleProp, StyleSheet, ViewProps, ViewStyle } from "react-native";
import { ThemedView } from "./themed-view";

type Props = ViewProps & {
  style?: StyleProp<ViewStyle>;
};

export default function Card({ children, style, ...props }: Props) {
  return (
    <ThemedView style={[styles.card, style]} {...props}>
      {children}
    </ThemedView>
  );
}

const styles = StyleSheet.create({
  card: {
    borderWidth: 1,
    borderRadius: 8,
    padding: 15,
    marginBottom: 10,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 5,
    elevation: 3,
  },
});
