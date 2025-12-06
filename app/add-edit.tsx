import ActionButton from "@/components/main/ActionButton";
import PrimaryButton from "@/components/main/PrimaryButton";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { globalColors } from "@/constants/global-colors";
import { AppDispatch, RootState } from "@/store";
import { useAppSelector } from "@/store/hooks";
import { addPlaceWithBackend, updatePlaceWithBackend } from "@/store/placeBackendThunks";
import { selectImagesById, selectPlaceById, selectPlaceUserId } from "@/store/placeSelectors";
import { addLocalImages, addPlace, markImageDeleted, updatePlace } from "@/store/placeSlice";
import { savePlacesAsync } from "@/store/placeThunks";
import { globalStyles } from "@/styles/globalStyles";
import { LocalImage } from "@/types/place";
import { exitToPreviousOrHome } from "@/utils/navigation";
import FontAwesome6 from "@expo/vector-icons/FontAwesome6";
import { randomUUID } from "expo-crypto";
import { Image } from "expo-image";
import * as ImagePicker from "expo-image-picker";
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
import { shallowEqual, useDispatch, useSelector } from "react-redux";

function isEmptyInput(value: string): boolean {
  return value.trim() === "";
}

export async function userCurrentLocation() {
  let { status } = await Location.requestForegroundPermissionsAsync();

  if (status !== "granted") {
    Alert.alert("Location Permission denied");
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

  const savedImages = useAppSelector((state: RootState) => {
    return typeof id === "string" ? selectImagesById(state, id) : [];
  }, shallowEqual);

  // Create states
  const [name, setName] = useState<string>(place?.name || "");
  const [category, setCategory] = useState<string>(place?.category || "");
  const [location, setLocation] = useState<string>(place?.location || "");
  const [note, setNote] = useState<string>(place?.note || "");
  const [newImages, setNewImages] = useState<LocalImage[]>([]);
  const [deletedImageIds, setDeletedImageIds] = useState<string[]>([]);

  const MEDIA_TYPE_LIBRARY = "library";
  const MEDIA_TYPE_CAMERA = "camera";
  const imagePickerOptions = {
    mediaTypes: ["images"],
    allowsEditing: true,
    aspect: [4, 3] as [number, number],
    quality: 1,
  };

  const deletedSet = useMemo(() => new Set(deletedImageIds), [deletedImageIds]);

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

  const grantImagePermissions = async (type: string) => {
    if (type === MEDIA_TYPE_LIBRARY) {
      const { status } = await ImagePicker.requestMediaLibraryPermissionsAsync();
      if (status !== "granted") {
        Alert.alert("Permission required", "Permission to access the media library is required.");
        return false;
      }
    } else if (type === MEDIA_TYPE_CAMERA) {
      const { status } = await ImagePicker.requestCameraPermissionsAsync();
      if (status !== "granted") {
        Alert.alert("Permission required", "Permission to access the camera is required.");
        return false;
      }
    }
    return true;
  };

  function handleImagePickerResult(result: ImagePicker.ImagePickerResult) {
    if (result.canceled) return;
    const asset = result.assets?.[0];
    if (!asset?.uri) return;

    const newImage: LocalImage = {
      id: randomUUID(),
      uri: asset.uri,
      saved: false,
    };
    setNewImages([...newImages, newImage]);
  }

  const pickImage = async () => {
    const permissionResult = await grantImagePermissions(MEDIA_TYPE_LIBRARY);

    if (!permissionResult) {
      return;
    }

    let result = await ImagePicker.launchImageLibraryAsync(imagePickerOptions);

    handleImagePickerResult(result);
  };

  const takePhoto = async () => {
    const permissionResult = await grantImagePermissions(MEDIA_TYPE_CAMERA);

    if (!permissionResult) {
      return;
    }

    let result = await ImagePicker.launchCameraAsync(imagePickerOptions);

    handleImagePickerResult(result);
  };

  const handleDeleteImage = (item: LocalImage) => {
    if (item.saved) {
      // delete saved images
      setDeletedImageIds([...deletedImageIds, item.id]);
    } else {
      // remove just captured but unsaved images
      setNewImages(newImages.filter((img) => img.id !== item.id));
    }
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
        setNewImages([]);
        setDeletedImageIds([]);
      }
    } else {
      placeId = randomUUID();
      const newPlace = {
        id: placeId,
        name,
        category,
        location,
        note,
      };
      dispatch(addPlace({ place: newPlace, images: newImages }));
      setNewImages([]);
    }

    if (userId) {
      await dispatch(savePlacesAsync(userId)).catch((err) =>
        console.error("AsyncStorage save failed:", err),
      );
    }

    if (placeId) {
      try {
        if (place) {
          // update with backend
          await dispatch(updatePlaceWithBackend({ placeId })).unwrap();
        } else {
          // add with backend
          await dispatch(addPlaceWithBackend({ placeId })).unwrap();
        }
      } catch (err) {
        console.error("Backend sync failed:", err);
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
          <ThemedView style={styles.previewImagesContainer}>
            {[...savedImages.filter((img) => !deletedSet.has(img.id)), ...newImages].map(
              (item, index) => (
                <ThemedView key={item.id} style={styles.imagePreviewContainer}>
                  <Image
                    source={{ uri: item.uri }}
                    style={styles.image}
                    accessibilityRole="image"
                    accessibilityLabel={`Place photo ${index + 1}${name ? ` for ${name}` : ""}`}
                  />
                  <Pressable
                    onPress={() => handleDeleteImage(item)}
                    style={styles.deleteImageButton}
                    accessibilityLabel="Delete image"
                    accessibilityRole="button"
                  >
                    <FontAwesome6 name="trash-can" size={24} color="red" />
                  </Pressable>
                </ThemedView>
              ),
            )}
          </ThemedView>

          <ThemedView style={styles.mediaButtonsRow}>
            <ActionButton
              variant="primary"
              style={[styles.button, styles.mediaButton]}
              onPress={pickImage}
              accessibilityLabel="Pick Images"
            >
              <FontAwesome6 name="image" size={20} color={globalColors.black} />
            </ActionButton>
            <ActionButton
              variant="primary"
              style={[styles.button, styles.mediaButton, styles.mediaButtonTrailing]}
              onPress={takePhoto}
              accessibilityLabel="Take Photo"
            >
              <FontAwesome6 name="camera" size={20} color={globalColors.black} />
            </ActionButton>
          </ThemedView>

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
  previewImagesContainer: {
    flexDirection: "row",
    flexWrap: "wrap",
  },
  mediaButtonsRow: {
    flexDirection: "row",
  },
  mediaButton: {
    flex: 1,
    marginRight: 10,
  },
  mediaButtonTrailing: {
    marginRight: 0,
  },
  imagePreviewContainer: { position: "relative", margin: 5 },
  image: { width: 100, height: 100, borderRadius: 8 },
  deleteImageButton: {
    position: "absolute",
    top: 4,
    right: 4,
    backgroundColor: "white",
    padding: 6,
    borderRadius: 20,
  },
});
