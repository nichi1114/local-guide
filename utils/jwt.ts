import { jwtDecode } from 'jwt-decode';

type JwtPayload = {
  exp?: number;
};

const EXPIRY_BUFFER_SECONDS = 60;

/**
 * Returns true when the provided JWT exists and its exp claim is still in the future.
 */

export function isJwtValid(token?: string | null): boolean {
  if (!token) {
    return false;
  }

  try {
    const payload = jwtDecode<JwtPayload>(token);
    if (!payload.exp) {
      return false;
    }

    const nowInSeconds = Date.now() / 1000;
    return payload.exp > nowInSeconds + EXPIRY_BUFFER_SECONDS;
  } catch {
    return false;
  }
}
