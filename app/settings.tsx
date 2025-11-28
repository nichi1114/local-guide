import ActionButton from "@/components/main/ActionButton";
import { ThemedText } from "@/components/themed-text";
import { ThemedView } from "@/components/themed-view";
import { globalColors } from "@/constants/global-colors";
import { selectAuthSession } from "@/store/authSelectors";
import { clearAuthSession } from "@/store/authSlice";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { globalStyles } from "@/styles/globalStyles";
import * as Notifications from "expo-notifications";
import { useRouter } from "expo-router";
import { useEffect, useState } from "react";
import { Alert, StyleSheet, Switch } from "react-native";

export default function SettingsScreen() {
  const dispatch = useAppDispatch();
  const router = useRouter();
  const session = useAppSelector(selectAuthSession);

  const [notificationsEnabled, setNotificationsEnabled] = useState(false);
  const [isCheckingNotifications, setIsCheckingNotifications] = useState(true);
  const [isUpdatingNotifications, setIsUpdatingNotifications] = useState(false);

  useEffect(() => {
    const fetchPermissions = async () => {
      try {
        const permissions = await Notifications.getPermissionsAsync();
        setNotificationsEnabled(permissions.granted);
      } catch (error) {
        console.warn("Failed to read notification permissions", error);
      } finally {
        setIsCheckingNotifications(false);
      }
    };

    fetchPermissions();
  }, []);

  const handleLogout = async () => {
    try {
      await Notifications.cancelAllScheduledNotificationsAsync();
    } catch (error) {
      console.warn("Failed to cancel notifications during logout", error);
    }
    await dispatch(clearAuthSession()).unwrap();
    router.replace("/login");
  };

  const handleToggleNotifications = async (value: boolean) => {
    setIsUpdatingNotifications(true);
    if (!value) {
      await Notifications.cancelAllScheduledNotificationsAsync();
      setNotificationsEnabled(false);
      setIsUpdatingNotifications(false);
      return;
    }

    try {
      const requested = await Notifications.requestPermissionsAsync();
      if (!requested.granted) {
        Alert.alert(
          "Notifications disabled",
          "Enable notifications in your device settings to receive reminders.",
        );
        setNotificationsEnabled(false);
        return;
      }

      await Notifications.cancelAllScheduledNotificationsAsync();
      await Notifications.scheduleNotificationAsync({
        content: {
          title: "Daily Reminder",
          body: "Discover something new? Log your place of interest!",
        },
        trigger: {
          hour: 9,
          minute: 0,
          repeats: true,
        },
      });
      setNotificationsEnabled(true);
    } catch (error) {
      console.warn("Failed to update notification preference", error);
      Alert.alert("Error", "We could not update notification settings. Please try again.");
      setNotificationsEnabled(false);
    } finally {
      setIsUpdatingNotifications(false);
    }
  };

  const userName = session?.user?.name ?? "Local Guide user";
  const userEmail = session?.user?.email ?? "Email not available";

  return (
    <ThemedView style={globalStyles.container}>
      <ThemedView style={styles.card}>
        <ThemedText type="subtitle">Account</ThemedText>
        <ThemedText type="defaultSemiBold">{userName}</ThemedText>
        <ThemedText style={styles.subdued}>{userEmail}</ThemedText>
      </ThemedView>

      <ThemedView style={styles.card}>
        <ThemedText type="subtitle">Notifications</ThemedText>
        <ThemedView style={styles.row}>
          <ThemedView style={styles.textBlock}>
            <ThemedText type="defaultSemiBold">Daily reminder</ThemedText>
            <ThemedText style={styles.helper}>
              Get a nudge to log new places every morning.
            </ThemedText>
          </ThemedView>
          <Switch
            value={notificationsEnabled}
            onValueChange={handleToggleNotifications}
            disabled={isCheckingNotifications || isUpdatingNotifications}
            trackColor={{ false: globalColors.border, true: globalColors.primary }}
            thumbColor={notificationsEnabled ? globalColors.black : globalColors.white}
            accessibilityRole="switch"
            accessibilityLabel="Toggle daily reminder notifications"
            accessibilityState={{
              checked: notificationsEnabled,
              disabled: isCheckingNotifications || isUpdatingNotifications,
            }}
          />
        </ThemedView>
      </ThemedView>

      <ActionButton variant="danger" onPress={handleLogout} style={styles.logoutButton}>
        Log Out
      </ActionButton>
    </ThemedView>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: "transparent",
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    gap: 10,
    borderWidth: 1,
    borderColor: globalColors.border,
  },
  row: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  textBlock: {
    flex: 1,
    gap: 4,
  },
  helper: {
    color: globalColors.placeholder,
  },
  subdued: {
    color: globalColors.textPrimary,
    opacity: 0.8,
  },
  logoutButton: {
    marginTop: 8,
  },
});
