import { Redirect } from "expo-router";

// Catches the Google OAuth deep link so we do not hit Expo Router's 404 screen.
export default function OAuthRedirectScreen() {
  return <Redirect href="/" />;
}
