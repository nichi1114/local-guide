import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { globalColors } from "@/constants/global-colors";
import { addPlace, selectPlaceById, updatePlace } from "@/store/placeSlice";
import { savePlacesAsync } from "@/store/placeThunks";
import { useLocalSearchParams, useRouter } from "expo-router";
import { useEffect, useState } from "react";
import { Alert, StyleSheet, TextInput } from "react-native";
import { useDispatch, useSelector } from "react-redux";
import PrimaryButton from "../components/main/PrimaryButton";
import { AppDispatch, RootState, store } from "../store";
import { globalStyles } from "../styles/globalStyles";

export default function AddEditScreen() {
  function isEmptyInput(value: string): boolean {
    return value.trim() === "";
  }

  const router = useRouter();
  const { id } = useLocalSearchParams();

  const dispatch = useDispatch<AppDispatch>();

  const place =
    typeof id === "string"
      ? useSelector((state: RootState) => selectPlaceById(state, id))
      : undefined;

  // Create states
  const [name, setName] = useState<string>(place?.name || "");
  const [category, setCategory] = useState<string>(place?.category || "");
  const [location, setLocation] = useState<string>(place?.location.toString() || "");
  const [note, setNote] = useState<string>(place?.note.toString() || "");

  useEffect(() => {
    if (place) {
      setName(place.name);
      setCategory(place.category);
      setLocation(place.location);
      setNote(place.note);
    }
  }, [place]);

  const handleSubmit = () => {
    // Validate inputs
    if (isEmptyInput(name)) {
      Alert.alert("Error", "Please enter a name");
      return;
    }

    // todo fetch from google api later
    if (isEmptyInput(category)) {
      Alert.alert("Error", "Please enter a category");
      return;
    }

    if (!isEmptyInput(location)) {
      // todo Expo location
      Alert.alert("Error", "Please enter a location");
      return;
    }

    // note can be empty

    // save to storage
    if (place) {
      // Dispatch updatePlace
      if (typeof id === "string") {
        dispatch(
          updatePlace({
            id: id,
            updated: {
              name,
              category,
              location,
              note,
            },
          }),
        );
      }
    } else {
      // Dispatch addPlace
      dispatch(
        addPlace({
          name,
          category,
          location,
          note,
        }),
      );
    }

    const { userId, places } = store.getState().poi;
    if (userId) {
      dispatch(savePlacesAsync({ userId, places }));
    }

    // Navigate back to Home ("/")
    router.push("/");
  };

  return (
    <ThemedView style={globalStyles.container}>
      <ThemedText type="title">{place ? "Edit Place" : "Add Place"}</ThemedText>

      <TextInput
        style={styles.input}
        placeholder="Name"
        value={name}
        onChangeText={setName}
        placeholderTextColor={globalColors.placeholder}
      />

      <TextInput
        style={styles.input}
        placeholder="Category (e.g. Cafe, Park)"
        value={category}
        onChangeText={setCategory}
        placeholderTextColor={globalColors.placeholder}
      />

      <TextInput
        style={styles.input}
        placeholder="Location"
        value={location}
        onChangeText={setLocation}
        placeholderTextColor={globalColors.placeholder}
      />

      <TextInput
        style={styles.input}
        placeholder="Note (optional)"
        value={note}
        onChangeText={setNote}
        placeholderTextColor={globalColors.placeholder}
      />

      <PrimaryButton onPress={handleSubmit} testID="add-button">
        {place ? "Update Place" : "Add Place"}
      </PrimaryButton>
    </ThemedView>
  );
}

const styles = StyleSheet.create({
  input: {
    backgroundColor: globalColors.white,
    borderWidth: 1,
    borderColor: globalColors.border,
    borderRadius: 8,
    padding: 10,
    marginBottom: 10,
    fontSize: 16,
    color: globalColors.textPrimary,
  },
});
