// Reuse the DetailsCard component implementation from Assignment.
import { LocalImage, Place } from "@/types/place";
import { Image } from "expo-image";
import { StyleSheet } from "react-native";
import { ThemedText } from "../themed-text";
import { ThemedView } from "../themed-view";
import Card from "./Card";

type Props = {
  place: Place;
  images: LocalImage[];
};

export default function DetailsCard({ place, images }: Props) {
  return (
    <Card>
      <ThemedText style={styles.text}>Name: {place.name}</ThemedText>
      <ThemedText style={styles.text}>Category: {place.category}</ThemedText>
      <ThemedText style={styles.text}>Location: {place.location}</ThemedText>
      {place.note ? <ThemedText style={styles.text}>Note: {place.note}</ThemedText> : null}
      <ThemedView
        style={{
          flexDirection: "row",
          flexWrap: "wrap",
        }}
      >
        {images.map((item) => (
          <ThemedView key={item.id} style={{ position: "relative", margin: 5 }}>
            <Image
              source={{ uri: item.uri }}
              style={{ width: 200, height: 200, borderRadius: 8 }}
            />
          </ThemedView>
        ))}
      </ThemedView>
    </Card>
  );
}

const styles = StyleSheet.create({
  text: {
    fontSize: 18,
    marginVertical: 5,
  },
});
