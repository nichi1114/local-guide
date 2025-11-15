// Reuse the PrimaryButton component implementation from Assignment.
import { Pressable, StyleProp, StyleSheet, Text, ViewStyle } from "react-native";

type Props = {
  onPress: () => void;
  children: string;
  style?: StyleProp<ViewStyle>;
  testID?: string | undefined;
};

export default function PrimaryButton({ onPress, children, style, testID }: Props) {
  return (
    <Pressable
      // Apply styles.button, dynamic background color when pressed, and any additional style from props
      style={({ pressed }) => [
        styles.button,
        {
          //todo backgroundColor: pressed ? colors.primaryPressed : colors.primary,
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

const styles = StyleSheet.create({
  button: {
    padding: 12,
    borderRadius: 10,
    alignItems: "center",
    justifyContent: "center",
    marginTop: 10,
  },
  text: {
    fontSize: 16,
    fontWeight: "bold",
  },
});
