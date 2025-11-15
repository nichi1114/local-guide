import PlaceListItem from "@/components/main/PlaceListItem";
import PrimaryButton from "@/components/main/PrimaryButton";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { selectPlaces, selectUserId } from "@/store/placeSlice";
import { loadPlacesAsync } from "@/store/placeThunks";
import { globalStyles } from "@/styles/globalStyles";
import { useRouter } from "expo-router";
import { useEffect } from "react";
import { FlatList, StyleSheet } from "react-native";

export default function HomeScreen() {
  const router = useRouter();
  const dispatch = useAppDispatch();
  const userId = useAppSelector(selectUserId);
  const places = useAppSelector(selectPlaces);

  useEffect(() => {
    if (userId) {
      dispatch(loadPlacesAsync(userId));
    }
  }, [userId]);

  return (
    <ThemedView style={globalStyles.container} testID="container">
      <PrimaryButton onPress={() => router.push("/add-edit")}>+</PrimaryButton>

      <ThemedView style={styles.buttonSpacer} />
      {!places || places.length === 0 ? (
        <ThemedText type="defaultSemiBold">
          You haven't added any places yet. Click '+' to get started!
        </ThemedText>
      ) : null}
      <FlatList
        data={places}
        keyExtractor={(item) => item.id}
        renderItem={({ item }) => <PlaceListItem place={item} />}
        contentContainerStyle={styles.list}
        testID="place-list"
      />
    </ThemedView>
  );
}

const styles = StyleSheet.create({
  list: {
    paddingBottom: 20,
  },
  buttonSpacer: {
    height: 20,
  },
});
