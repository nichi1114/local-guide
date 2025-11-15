import ActionButton from "@/components/main/ActionButton";
import PrimaryButton from "@/components/main/PrimaryButton";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { globalColors } from "@/constants/global-colors";
import { AppDispatch, RootState } from "@/store";
import { useAppSelector } from "@/store/hooks";
import { addPlace, selectPlaceById, selectPlaceUserId, updatePlace } from "@/store/placeSlice";
import { savePlacesAsync } from "@/store/placeThunks";
import { globalStyles } from "@/styles/globalStyles";
import * as Location from "expo-location";
import { useLocalSearchParams, useRouter } from "expo-router";
import { useEffect, useState } from "react";
import {
  Alert,
  KeyboardAvoidingView,
  Platform,
  ScrollView,
  StyleSheet,
  TextInput,
} from "react-native";
import { useDispatch, useSelector } from "react-redux";

function isEmptyInput(value: string): boolean {
  return value.trim() === "";
}

export async function userCurrentLocation() {
  let { status } = await Location.requestForegroundPermissionsAsync();

  if (status !== "granted") {
    Alert.alert("Permission denied");
    return;
  }

  const location: Location.LocationObject = await Location.getCurrentPositionAsync({});
  return {
    la: location.coords.latitude,
    long: location.coords.longitude,
  };
}

export async function convertCoordsToAddress(la: number, long: number) {
  const result = await Location.reverseGeocodeAsync({ latitude: la, longitude: long });
  if (!result || result.length === 0) {
    return "Unknown location";
  }

  const primaryResult = result[0];

  const address = [
    primaryResult.name,
    primaryResult.street,
    primaryResult.city,
    primaryResult.region,
    primaryResult.postalCode,
    primaryResult.country,
  ]
    .filter(Boolean)
    .join(", ");

  return address;
}

export default function AddEditScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams();
  const userId = useAppSelector(selectPlaceUserId);

  const dispatch = useDispatch<AppDispatch>();

  const place = useSelector((state: RootState) =>
    typeof id === "string" ? selectPlaceById(state, id) : undefined,
  );

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

  const handleUseCurrentLocation = async () => {
    const coords = await userCurrentLocation();
    if (!coords) return;

    const address = await convertCoordsToAddress(coords.la, coords.long);
    console.log(address);
    setLocation(address);
  };

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
      dispatch(savePlacesAsync(userId));
    }

    router.push("/");
  };

  return (
    <KeyboardAvoidingView
      style={{ flex: 1 }}
      behavior={Platform.OS === "ios" ? "padding" : undefined}
    >
      <ScrollView>
        <ThemedView style={globalStyles.container}>
          <ThemedText type="subtitle">{place ? "Edit Place" : "Add Place"}</ThemedText>
          <ThemedView style={styles.buttonSpacer} />

          <ThemedText>Name:</ThemedText>
          <TextInput
            style={styles.input}
            placeholder="Name"
            value={name}
            onChangeText={setName}
            placeholderTextColor={globalColors.placeholder}
          />

          <ThemedText>Category:</ThemedText>
          <TextInput
            style={styles.input}
            placeholder="Category (e.g. Cafe, Park)"
            value={category}
            onChangeText={setCategory}
            placeholderTextColor={globalColors.placeholder}
          />

          <ThemedText>Location:</ThemedText>
          <TextInput
            style={styles.input}
            placeholder="Location"
            value={location}
            onChangeText={setLocation}
            placeholderTextColor={globalColors.placeholder}
          />
          <ActionButton
            variant="primary"
            style={styles.useLocationButton}
            onPress={handleUseCurrentLocation}
            testID="use-current-location-button"
          >
            Use Current Location
          </ActionButton>
          <ThemedText>Note:</ThemedText>
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
      </ScrollView>
    </KeyboardAvoidingView>
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
  useLocationButton: {
    ...baseInput,
    alignItems: "center",
    justifyContent: "center",
  },
  buttonSpacer: {
    height: 10,
  },
});
