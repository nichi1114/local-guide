// Reuse the DetailsCard component implementation from Assignment.
import { Place } from "@/types/place";
import { StyleSheet } from "react-native";
import { ThemedText } from "../themed-text";
import Card from "./Card";

type Props = {
  place: Place;
};

export default function DetailsCard({ place }: Props) {
  return (
    <Card>
      <ThemedText style={styles.text}>Name: {place.name}</ThemedText>
      <ThemedText style={styles.text}>Category: {place.category}</ThemedText>
      <ThemedText style={styles.text}>Location: {place.location}</ThemedText>
      {place.note ? <ThemedText style={styles.text}>Note: {place.note}</ThemedText> : null}
    </Card>
  );
}

const styles = StyleSheet.create({
  text: {
    fontSize: 18,
    marginVertical: 5,
  },
});
