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
      <ThemedView style={styles.previewImagesContainer}>
        {images.map((item, index) => (
          <ThemedView key={item.id} style={styles.imagePreviewContainer}>
            <Image
              source={{ uri: item.uri }}
              style={styles.image}
              accessibilityRole="image"
              accessibilityLabel={`Place photo ${index + 1}${place.name ? ` for ${place.name}` : ""}`}
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
  previewImagesContainer: {
    flexDirection: "row",
    flexWrap: "wrap",
  },
  imagePreviewContainer: { position: "relative", margin: 5 },
  image: { width: 200, height: 200, borderRadius: 8 },
});
