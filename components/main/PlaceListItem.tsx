import { AppDispatch, store } from "@/store";
import { deletePlace } from "@/store/placeSlice";
import { savePlacesAsync } from "@/store/placeThunks";
import { Place } from "@/types/place";
import { useRouter } from "expo-router";
import React from "react";
import { Pressable, StyleSheet } from "react-native";
import { useDispatch } from "react-redux";
import { ThemedText } from "../themed-text";
import { ThemedView } from "../themed-view";
import ActionButton from "./ActionButton";
import Card from "./Card";

type Props = {
  place: Place;
};

export default function PlaceListItem({ place }: Props) {
  const router = useRouter();
  const dispatch = useDispatch<AppDispatch>();

  const handleDelete = () => {
    // udpate state
    dispatch(deletePlace(place.id));

    // save to storage
    const { userId, places } = store.getState().poi;
    if (userId) {
      dispatch(savePlacesAsync({ userId, places }));
    }
  };

  return (
    // Apply styles.card
    <Card style={styles.card}>
      <Pressable
        style={({ pressed }) => [styles.content, { opacity: pressed ? 0.6 : 1 }]}
        onPress={() => router.push(`/places/${place.id}`)}
      >
        <ThemedText style={styles.text}>{place.name}</ThemedText>

        <ThemedText style={styles.text}>Category: {place.category}</ThemedText>

        <ThemedText style={styles.text}>Location: {place.location}</ThemedText>
      </Pressable>

      <ThemedView style={styles.buttons}>
        <ActionButton variant="primary" onPress={() => router.push(`/add-edit?id=${place.id}`)}>
          Edit
        </ActionButton>
        <ActionButton variant="danger" onPress={handleDelete}>
          Delete
        </ActionButton>
      </ThemedView>
    </Card>
  );
}

const styles = StyleSheet.create({
  card: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
  },
  content: {
    flex: 1,
    flexDirection: "column",
  },
  text: {
    fontSize: 16,
    marginVertical: 3,
  },
  buttons: {
    flexDirection: "column",
    gap: 10,
  },
});
