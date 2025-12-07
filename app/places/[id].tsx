import ActionButton from "@/components/main/ActionButton";
import DetailsCard from "@/components/main/DetailsCard";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { RootState } from "@/store";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { deletePlaceWithBackend } from "@/store/placeBackendThunks";
import { selectImagesById, selectPlaceById, selectPlaceUserId } from "@/store/placeSelectors";
import { deletePlace } from "@/store/placeSlice";
import { savePlacesAsync } from "@/store/placeThunks";
import { globalStyles } from "@/styles/globalStyles";
import { exitToPreviousOrHome } from "@/utils/navigation";
import { useLocalSearchParams, useRouter } from "expo-router";
import { Alert, ScrollView, StyleSheet } from "react-native";
import { useSelector } from "react-redux";

export default function DetailsScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams();
  const userId = useAppSelector(selectPlaceUserId);

  const dispatch = useAppDispatch();

  const place = useSelector((state: RootState) =>
    typeof id === "string" ? selectPlaceById(state, id) : undefined,
  );

  const savedImages = useSelector((state: RootState) =>
    typeof id === "string" ? selectImagesById(state, id) : [],
  );

  if (!place) {
    return (
      <ThemedView style={globalStyles.container}>
        <ThemedText type="title">Place Not Found</ThemedText>
      </ThemedView>
    );
  }

  const handleDelete = async () => {
    if (typeof id === "string") {
      // backend
      try {
        await dispatch(deletePlaceWithBackend({ placeId: id })).unwrap();
        dispatch(deletePlace(id));

        if (userId) {
          // local
          await dispatch(savePlacesAsync(userId)).catch((err) =>
            console.error("AsyncStorage save failed:", err),
          );
        }
        // Navigate back without stacking another Home screen
        exitToPreviousOrHome(router, "/");
      } catch (err) {
        console.error("Backend delete place failed:", err);
        Alert.alert("Delete Failed", "We couldn't delete this place. Please try again.");
      }
    }
  };

  return (
    <ScrollView>
      <ThemedView style={globalStyles.container}>
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

        <DetailsCard place={place} images={savedImages} />
      </ThemedView>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  // Row style for buttons
  buttons: {
    flexDirection: "row",
    justifyContent: "space-between",
    marginBottom: 10,
    gap: 10,
  },
  // Individual button style
  button: { flex: 1 },
});
