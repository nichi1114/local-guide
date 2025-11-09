import { StyleSheet } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useEffect } from 'react';
import { useRouter } from 'expo-router';

import { GoogleSignInCard } from '@/components/google-sign-in-card';
import { useAppSelector } from '@/store/hooks';
import { selectHasValidToken, selectIsHydratingAuth } from '@/store/authSelectors';

export default function LoginScreen() {
  const hasValidToken = useAppSelector(selectHasValidToken);
  const isHydrating = useAppSelector(selectIsHydratingAuth);
  const router = useRouter();

  useEffect(() => {
    if (!isHydrating && hasValidToken) {
      router.replace('/');
    }
  }, [hasValidToken, isHydrating, router]);

  return (
    <SafeAreaView style={styles.container}>
      <GoogleSignInCard />
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 24,
    justifyContent: 'center',
  },
});
