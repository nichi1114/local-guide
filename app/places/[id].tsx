import ActionButton from "@/components/main/ActionButton";
import DetailsCard from "@/components/main/DetailsCard";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { RootState } from "@/store";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { deletePlace, selectPlaceById, selectPlaces, selectUserId } from "@/store/placeSlice";
import { savePlacesAsync } from "@/store/placeThunks";
import { globalStyles } from "@/styles/globalStyles";
import { useLocalSearchParams, useRouter } from "expo-router";
import { StyleSheet } from "react-native";
import { useSelector } from "react-redux";

export default function DetailsScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams();
  const userId = useAppSelector(selectUserId);
  const places = useAppSelector(selectPlaces);

  const dispatch = useAppDispatch();

  const place = useSelector((state: RootState) =>
    typeof id === "string" ? selectPlaceById(state, id) : undefined,
  );

  if (!place) {
    return (
      <ThemedView style={globalStyles.container}>
        <ThemedText type="title">Place Not Found</ThemedText>
      </ThemedView>
    );
  }

  const handleDelete = () => {
    if (typeof id === "string") {
      dispatch(deletePlace(id));

      if (userId) {
        dispatch(savePlacesAsync({ userId, places }));
      }
    }
    // Navigate back to Home ("/")
    router.push("/");
  };

  return (
    <ThemedView style={globalStyles.container}>
      <DetailsCard place={place} />

      <ThemedView style={styles.buttons}>
        <ActionButton
          variant="primary"
          onPress={() => router.push(`/add-edit?id=${place.id}`)}
          style={styles.button}
          testID="edit-button"
        >
          Edit
        </ActionButton>
        <ActionButton
          variant="danger"
          onPress={handleDelete}
          style={styles.button}
          testID="delete-button"
        >
          Delete
        </ActionButton>
      </ThemedView>
    </ThemedView>
  );
}

const styles = StyleSheet.create({
  // Row style for buttons
  buttons: {
    flexDirection: "row",
    justifyContent: "space-between",
    marginTop: 10,
    gap: 10,
  },
  // Individual button style
  button: { flex: 1 },
});
