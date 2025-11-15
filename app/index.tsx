import PlaceListItem from "@/components/PlaceListItem";
import PrimaryButton from "@/components/PrimaryButton";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { useRouter } from "expo-router";
import { FlatList, StyleSheet } from "react-native";
import { useSelector } from "react-redux";
import { RootState } from "../store/";

export default function HomeScreen() {
  const router = useRouter();

  const places = useSelector((state: RootState) => state.poi.places);

  return (
    <ThemedView testID="container">
      <ThemedText type="title">Home</ThemedText>
      <PrimaryButton onPress={() => router.push("/add-edit")}>Add</PrimaryButton>

      <ThemedView style={styles.buttonSpacer} />
      {!places || places.length === 0 ? (
        <ThemedText type="defaultSemiBold">
          You haven't added any places yet. Click 'Add' to get started!
        </ThemedText>
      ) : (
        <FlatList
          data={places}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => <PlaceListItem place={item} />}
          contentContainerStyle={styles.list}
          testID="activity-list"
        />
      )}
    </ThemedView>
  );
}

const styles = StyleSheet.create({
  // Define list padding (paddingBottom: 20)
  list: {
    paddingBottom: 20,
  },

  // Define button spacer (height: 20)
  buttonSpacer: {
    height: 20,
  },
});
