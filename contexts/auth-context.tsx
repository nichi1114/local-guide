import AsyncStorage from '@react-native-async-storage/async-storage';
import {
  ReactNode,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react';

import type { BackendLoginResponse } from '@/types/auth';
import { isJwtValid } from '@/utils/jwt';

type AuthContextValue = {
  session: BackendLoginResponse | null;
  isHydrating: boolean;
  saveSession: (session: BackendLoginResponse) => Promise<void>;
  clearSession: () => Promise<void>;
  hasValidToken: () => boolean;
};

const STORAGE_KEY = 'local-guide:auth-session';
const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<BackendLoginResponse | null>(null);
  const [isHydrating, setIsHydrating] = useState(true);

  useEffect(() => {
    const hydrate = async () => {
      try {
        const raw = await AsyncStorage.getItem(STORAGE_KEY);
        if (raw) {
          const parsed = JSON.parse(raw) as BackendLoginResponse;
          setSession(parsed);
        }
      } catch (storageError) {
        console.warn('Failed to hydrate auth session', storageError);
      } finally {
        setIsHydrating(false);
      }
    };

    hydrate();
  }, []);

  const saveSession = useCallback(async (nextSession: BackendLoginResponse) => {
    setSession(nextSession);
    try {
      await AsyncStorage.setItem(STORAGE_KEY, JSON.stringify(nextSession));
    } catch (storageError) {
      console.warn('Failed to persist auth session', storageError);
    }
  }, []);

  const clearSession = useCallback(async () => {
    setSession(null);
    try {
      await AsyncStorage.removeItem(STORAGE_KEY);
    } catch (storageError) {
      console.warn('Failed to clear auth session', storageError);
    }
  }, []);

  const hasValidToken = useCallback(() => isJwtValid(session?.jwt_token), [session]);

  const value = useMemo(
    () => ({
      session,
      isHydrating,
      saveSession,
      clearSession,
      hasValidToken,
    }),
    [session, isHydrating, saveSession, clearSession, hasValidToken]
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }

  return context;
}
