// Reuse the ActionButton component implementation from Assignment.
import { Pressable, StyleProp, StyleSheet, Text, ViewStyle } from "react-native";

type Variant = "primary" | "danger";

type Props = {
  onPress: () => void;
  children: string;
  variant: Variant;
  style?: StyleProp<ViewStyle>;
  testID?: string | undefined;
};

export default function ActionButton({ onPress, children, variant, style, testID }: Props) {
  const variantColors = {
    primary: { base: colors.primary, pressed: colors.primaryPressed },
    danger: { base: colors.danger, pressed: colors.dangerPressed },
  };

  return (
    <Pressable
      // Apply styles.button, dynamic background color based on variant & pressed state, and any additional style
      style={({ pressed }) => [
        styles.button,
        {
          backgroundColor: pressed ? variantColors[variant].pressed : variantColors[variant].base,
        },
        style,
      ]}
      onPress={onPress}
      testID={testID}
      accessibilityRole="button"
    >
      {/* Apply styles.text */}
      <Text style={styles.text}>{children}</Text>
    </Pressable>
  );
}

const colors = {
  primary: "#ffd900ff",
  primaryPressed: "#b89f0fff",
  danger: "#dc3545",
  dangerPressed: "#a71d31",
  white: "#fff",
};

const styles = StyleSheet.create({
  button: {
    padding: 8,
    borderRadius: 6,
    alignItems: "center",
    justifyContent: "center",
    minWidth: 60,
  },
  text: {
    fontSize: 16,
    fontWeight: "bold",
  },
});
