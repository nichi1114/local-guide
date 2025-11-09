export type BackendUser = {
  id: string;
  email?: string | null;
  name?: string | null;
  avatar_url?: string | null;
};

export type BackendLoginResponse = {
  user: BackendUser;
  jwt_token: string;
};
