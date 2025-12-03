import ActionButton from "@/components/main/ActionButton";
import PrimaryButton from "@/components/main/PrimaryButton";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { globalColors } from "@/constants/global-colors";
import { AppDispatch, RootState } from "@/store";
import { useAppSelector } from "@/store/hooks";
import { addPlaceWithBackend, updatePlaceWithBackend } from "@/store/placeBackendThunks";
import {
  makeSelectImagesByPlaceId,
  selectPlaceById,
  selectPlaceUserId,
} from "@/store/placeSelectors";
import { addLocalImages, addPlace, markImageDeleted, updatePlace } from "@/store/placeSlice";
import { savePlacesAsync } from "@/store/placeThunks";
import { globalStyles } from "@/styles/globalStyles";
import { LocalImage } from "@/types/place";
import { exitToPreviousOrHome } from "@/utils/navigation";
import FontAwesome6 from "@expo/vector-icons/FontAwesome6";
import { useCameraPermissions } from "expo-camera";
import { randomUUID } from "expo-crypto";
import { Image } from "expo-image";
import * as Location from "expo-location";
import { useLocalSearchParams, useRouter } from "expo-router";
import { useEffect, useMemo, useState } from "react";
import {
  Alert,
  KeyboardAvoidingView,
  Platform,
  Pressable,
  ScrollView,
  StyleSheet,
  TextInput,
} from "react-native";
import { useDispatch, useSelector } from "react-redux";
import CameraModal from "./camera-modal";

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

  const selectImagesByPlaceId = useMemo(
    () => (place ? makeSelectImagesByPlaceId(place.id) : null),
    [place],
  );

  const savedImages = useSelector((state: RootState) =>
    selectImagesByPlaceId ? selectImagesByPlaceId(state) : [],
  );

  // Create states
  const [name, setName] = useState<string>(place?.name || "");
  const [category, setCategory] = useState<string>(place?.category || "");
  const [location, setLocation] = useState<string>(place?.location || "");
  const [note, setNote] = useState<string>(place?.note || "");
  const [newImages, setNewImages] = useState<LocalImage[]>([]);
  const [deletedImageIds, setDeleteImagesIds] = useState<string[]>([]);
  const [permission, requestPermission] = useCameraPermissions();

  const [cameraVisible, setCameraVisible] = useState(false);

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
    setLocation(address);
  };

  const handleSubmit = async () => {
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
      Alert.alert("Error", "Please enter a location");
      return;
    }

    // note can be empty

    let placeId: string | undefined = place?.id;
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
        dispatch(markImageDeleted({ placeId: place.id, imageIds: deletedImageIds }));
        dispatch(addLocalImages({ placeId: place.id, images: newImages }));
      }
    } else {
      placeId = randomUUID();
      const newPlace = {
        id: placeId,
        name,
        category,
        location,
        note,
        newImages,
      };
      dispatch(addPlace({ place: newPlace, images: newImages }));
    }

    if (userId) {
      dispatch(savePlacesAsync(userId))
        .then(() => console.log("Saved to AsyncStorage"))
        .catch((err) => console.error("AsyncStorage save failed:", err));
    }

    if (placeId) {
      if (place) {
        // update with backend
        dispatch(updatePlaceWithBackend({ placeId }))
          .then(() => console.log("Update place to backend"))
          .catch((err) => {
            console.error("Backend update place failed:", err);
          });
      } else {
        // add with backend
        dispatch(addPlaceWithBackend({ placeId }))
          .then(() => console.log("Add place to backend"))
          .catch((err) => {
            console.error("Backend add place failed:", err);
          });
      }
    }

    exitToPreviousOrHome(router, "/");
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

          <ThemedText type="defaultSemiBold">Name:</ThemedText>
          <TextInput
            style={styles.input}
            placeholder="Name"
            value={name}
            onChangeText={setName}
            placeholderTextColor={globalColors.placeholder}
          />

          <ThemedText type="defaultSemiBold">Category:</ThemedText>
          <TextInput
            style={styles.input}
            placeholder="Category (e.g. Cafe, Park)"
            value={category}
            onChangeText={setCategory}
            placeholderTextColor={globalColors.placeholder}
          />

          <ThemedText type="defaultSemiBold">Location:</ThemedText>
          <TextInput
            style={styles.input}
            placeholder="Location"
            value={location}
            onChangeText={setLocation}
            placeholderTextColor={globalColors.placeholder}
          />
          <ActionButton
            variant="primary"
            style={styles.button}
            onPress={handleUseCurrentLocation}
            testID="use-current-location-button"
          >
            Use Current Location
          </ActionButton>

          <ThemedText type="defaultSemiBold">Images:</ThemedText>
          <ThemedView
            style={{
              flexDirection: "row",
              flexWrap: "wrap",
            }}
          >
            {[...savedImages.filter((img) => !deletedImageIds.includes(img.id)), ...newImages].map(
              (item) => (
                <ThemedView key={item.id} style={{ position: "relative", margin: 5 }}>
                  <Image
                    source={{ uri: item.uri }}
                    style={{ width: 100, height: 100, borderRadius: 8 }}
                  />
                  <Pressable
                    onPress={() => {
                      if (item.saved) {
                        setDeleteImagesIds([...deletedImageIds, item.id]);
                      } else {
                        setNewImages(newImages.filter((img) => img.id !== item.id));
                      }
                    }}
                    style={{
                      position: "absolute",
                      top: 4,
                      right: 4,
                      backgroundColor: "white",
                      padding: 6,
                      borderRadius: 20,
                    }}
                  >
                    <FontAwesome6 name="trash-can" size={24} color="red" />
                  </Pressable>
                </ThemedView>
              ),
            )}
          </ThemedView>

          {!permission || !permission.granted ? (
            <ThemedView style={{ flex: 1, justifyContent: "center", alignItems: "center" }}>
              <ThemedText>We need your permission to use the camera</ThemedText>
              <ActionButton variant="primary" style={styles.button} onPress={requestPermission}>
                Grant Permission
              </ActionButton>
            </ThemedView>
          ) : (
            <ActionButton
              variant="primary"
              style={styles.button}
              onPress={() => setCameraVisible(true)}
            >
              Add Photos
            </ActionButton>
          )}

          <CameraModal
            visible={cameraVisible}
            setImages={setNewImages}
            onClose={() => setCameraVisible(false)}
          />

          <ThemedText type="defaultSemiBold">Note:</ThemedText>
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
  button: {
    ...baseInput,
    alignItems: "center",
    justifyContent: "center",
  },
  buttonSpacer: {
    height: 10,
  },
});
