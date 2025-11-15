import PrimaryButton from "@/components/main/PrimaryButton";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { globalColors } from "@/constants/global-colors";
import { AppDispatch, RootState } from "@/store";
import { useAppSelector } from "@/store/hooks";
import {
  addPlace,
  selectPlaceById,
  selectPlaces,
  selectUserId,
  updatePlace,
} from "@/store/placeSlice";
import { savePlacesAsync } from "@/store/placeThunks";
import { globalStyles } from "@/styles/globalStyles";
import { useLocalSearchParams, useRouter } from "expo-router";
import { useEffect, useState } from "react";
import { Alert, StyleSheet, TextInput } from "react-native";
import { useDispatch, useSelector } from "react-redux";

export default function AddEditScreen() {
  function isEmptyInput(value: string): boolean {
    return value.trim() === "";
  }

  const router = useRouter();
  const { id } = useLocalSearchParams();
  const userId = useAppSelector(selectUserId);
  const places = useAppSelector(selectPlaces);

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

    if (isEmptyInput(location)) {
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
      dispatch(
        addPlace({
          name,
          category,
          location,
          note,
        }),
      );
    }

    if (userId) {
      dispatch(savePlacesAsync({ userId, places }));
    }

    router.push("/");
  };

  return (
    <ThemedView style={globalStyles.container}>
      <ThemedText type="subtitle">{place ? "Edit Place" : "Add Place"}</ThemedText>
      <ThemedView style={styles.buttonSpacer} />

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
        style={styles.largerInput}
        placeholder="Note (optional)"
        value={note}
        onChangeText={setNote}
        multiline={true}
        textAlignVertical="top"
        placeholderTextColor={globalColors.placeholder}
      />

      <PrimaryButton onPress={handleSubmit} testID="add-button">
        {place ? "Update Place" : "Add Place"}
      </PrimaryButton>
    </ThemedView>
  );
}

const baseInput = {
  backgroundColor: globalColors.white,
  borderWidth: 1,
  borderColor: globalColors.border,
  borderRadius: 8,
  padding: 10,
  marginBottom: 10,
  fontSize: 16,
  color: globalColors.textPrimary,
};

const styles = StyleSheet.create({
  input: baseInput,
  largerInput: {
    ...baseInput,
    height: 100,
  },
  buttonSpacer: {
    height: 10,
  },
});
