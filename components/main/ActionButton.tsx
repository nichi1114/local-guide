// Reuse the ActionButton component implementation from Assignment.
import { globalColors } from "@/constants/global-colors";
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
    primary: { base: globalColors.primary, pressed: globalColors.primaryPressed },
    danger: { base: globalColors.danger, pressed: globalColors.dangerPressed },
  };

  return (
    <Pressable
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
      <Text style={styles.text}>{children}</Text>
    </Pressable>
  );
}

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
