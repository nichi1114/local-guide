import PlaceListItem from "@/components/main/PlaceListItem";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { globalColors } from "@/constants/global-colors";
import { selectAuthSession } from "@/store/authSelectors";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { selectPlaces, setUserId } from "@/store/placeSlice";
import { loadPlacesAsync } from "@/store/placeThunks";
import { globalStyles } from "@/styles/globalStyles";
import * as Notifications from "expo-notifications";
import { useRouter } from "expo-router";
import { useEffect } from "react";
import { FlatList, Pressable, StyleSheet } from "react-native";

Notifications.setNotificationHandler({
  handleNotification: async () => ({
    shouldShowBanner: true,
    shouldShowList: true,
    shouldPlaySound: true,
    shouldSetBadge: false,
  }),
});

export default function HomeScreen() {
  const router = useRouter();
  const dispatch = useAppDispatch();
  const session = useAppSelector(selectAuthSession);
  const places = useAppSelector(selectPlaces);
  const HARDCODED_USER_ID = "test-user-id-123";

  useEffect(() => {
    if (session?.user?.id) {
      dispatch(setUserId(session.user.id));
      dispatch(loadPlacesAsync(session.user.id));
    } else {
      // Hardcoded test id
      dispatch(setUserId(HARDCODED_USER_ID));
      dispatch(loadPlacesAsync(HARDCODED_USER_ID));
    }
  }, [dispatch, session?.user?.id]);

  useEffect(() => {
    const subscription = Notifications.addNotificationResponseReceivedListener((response) => {
      console.log("User tapped notification:", response.notification.request.content);
    });
    return () => subscription.remove();
  }, []);

  return (
    <ThemedView style={globalStyles.container} testID="container">
      <ThemedView style={styles.row}>
        <Pressable
          style={({ pressed }) => [
            styles.topButton,
            {
              backgroundColor: pressed ? globalColors.primaryPressed : globalColors.primary,
            },
          ]}
          onPress={() => router.push("/add-edit")}
        >
          <ThemedText type="defaultSemiBold">Add</ThemedText>
        </Pressable>

        <Pressable
          style={({ pressed }) => [
            styles.topButton,
            {
              backgroundColor: pressed ? globalColors.primaryPressed : globalColors.primary,
            },
          ]}
          onPress={() => router.push("/settings")}
        >
          <ThemedText type="defaultSemiBold">Account</ThemedText>
        </Pressable>
      </ThemedView>

      <ThemedView style={styles.buttonSpacer} />
      {!places || places.length === 0 ? (
        <ThemedText type="defaultSemiBold">
          You haven&apos;t added any places yet. Click &apos;Add&apos; to get started!
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
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    width: "100%",
  },
  topButton: {
    padding: 8,
    borderRadius: 6,
    minWidth: 60,
    alignItems: "center",
    justifyContent: "center",
  },
});
