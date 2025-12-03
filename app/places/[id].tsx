import ActionButton from "@/components/main/ActionButton";
import DetailsCard from "@/components/main/DetailsCard";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { RootState } from "@/store";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { deletePlaceWithBackend } from "@/store/placeBackendThunks";
import { selectPlaceById, selectPlaceUserId } from "@/store/placeSelectors";
import { deletePlace } from "@/store/placeSlice";
import { savePlacesAsync } from "@/store/placeThunks";
import { globalStyles } from "@/styles/globalStyles";
import { exitToPreviousOrHome } from "@/utils/navigation";
import { useLocalSearchParams, useRouter } from "expo-router";
import { StyleSheet } from "react-native";
import { useSelector } from "react-redux";

export default function DetailsScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams();
  const userId = useAppSelector(selectPlaceUserId);

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
        // local
        dispatch(savePlacesAsync(userId))
          .then(() => console.log("Saved to AsyncStorage"))
          .catch((err) => console.error("AsyncStorage save failed:", err));
      }

      // backend
      dispatch(deletePlaceWithBackend({ placeId: id }))
        .then(() => console.log("Delete place with Backend"))
        .catch((err) => console.error("Backend delete place failed:", err));
    }
    // Navigate back without stacking another Home screen
    exitToPreviousOrHome(router, "/");
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
