import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { LocalImage } from "@/types/place";
import FontAwesome6 from "@expo/vector-icons/FontAwesome6";
import { CameraType, CameraView } from "expo-camera";
import { randomUUID } from "expo-crypto";
import { Image } from "expo-image";
import React, { useRef, useState } from "react";
import { Modal, Pressable, StyleSheet } from "react-native";

type CameraModalProps = {
  visible: boolean;
  setImages: React.Dispatch<React.SetStateAction<LocalImage[]>>;
  onClose: () => void;
};

export default function CameraModal({ visible, setImages, onClose }: CameraModalProps) {
  const [facing, setFacing] = useState<CameraType>("back");
  const [uri, setUri] = useState<string | null>(null);
  const cameraRef = useRef<CameraView>(null);

  const takePicture = async () => {
    const photo = await cameraRef.current?.takePictureAsync({ quality: 0.7, base64: false });
    if (photo?.uri) {
      console.log("Captured:", photo.uri);
      setUri(photo.uri);
    }
  };

  const retake = () => setUri(null);

  const confirm = () => {
    if (!uri) return;
    const id = randomUUID();
    const newImage: LocalImage = {
      id: id,
      uri: uri,
    };
    console.log("image id", id);
    setImages((prevImages) => [...prevImages, newImage]);
    setUri(null);
    onClose();
  };

  const cancel = () => {
    setUri(null);
    onClose();
  };

  const toggleCameraFacing = () => setFacing((cur) => (cur === "back" ? "front" : "back"));

  return (
    <Modal visible={visible} animationType="slide" transparent={false}>
      <ThemedView style={styles.container}>
        {!uri ? (
          <>
            <CameraView
              style={StyleSheet.absoluteFill}
              ref={cameraRef}
              mode="picture"
              facing={facing}
            />

            <Pressable style={styles.shutter} onPress={takePicture} />
            <Pressable style={styles.flipButton} onPress={toggleCameraFacing}>
              <FontAwesome6 name="rotate-left" size={32} color="white" />
            </Pressable>
            <Pressable style={styles.closeButton} onPress={cancel}>
              <ThemedText>Cancel</ThemedText>
            </Pressable>
          </>
        ) : (
          // Preview
          <ThemedView style={styles.previewContainer}>
            <ThemedView style={styles.previewImageContainer}>
              <Image
                source={{ uri }}
                contentFit="contain"
                style={{ width: "100%", height: "100%", aspectRatio: 1 }}
              />
            </ThemedView>

            <ThemedView style={styles.buttonRow}>
              <Pressable style={styles.previewButton} onPress={retake}>
                <ThemedText>Retake</ThemedText>
              </Pressable>
              <Pressable style={styles.previewButton} onPress={confirm}>
                <ThemedText>Confirm</ThemedText>
              </Pressable>
            </ThemedView>
          </ThemedView>
        )}
      </ThemedView>
    </Modal>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: "black" },
  shutter: {
    position: "absolute",
    bottom: 40,
    alignSelf: "center",
    width: 70,
    height: 70,
    borderRadius: 35,
    borderWidth: 5,
    borderColor: "white",
  },
  flipButton: { position: "absolute", top: 40, right: 20 },
  closeButton: { position: "absolute", top: 40, left: 20 },
  previewContainer: {
    flex: 1,
    justifyContent: "center",
    alignItems: "center",
    backgroundColor: "black",
  },
  previewImageContainer: { flex: 1, height: "100%" },
  buttonRow: { flexDirection: "row", justifyContent: "space-around", width: "100%", padding: 20 },
  previewButton: { backgroundColor: "white", padding: 15, borderRadius: 10 },
});
