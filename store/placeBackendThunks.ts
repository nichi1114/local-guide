import { API_BASE_URL } from "@/constants/env";
import { LocalImage } from "@/types/place";
import { createAsyncThunk } from "@reduxjs/toolkit";
import { RootState } from ".";
import { clearDeletedImages, markImagesSaved } from "./placeSlice";
import { savePlacesAsync } from "./placeThunks";

function getImageFileMeta(uri: string) {
  const filename = uri.split("/").pop() || "image.jpg";
  const ext = filename.split(".").pop();
  const type = ext ? `image/${ext}` : "image/jpeg";
  return { filename, type };
}

// Add a place to backend
export const addPlaceWithBackend = createAsyncThunk<
  void,
  { placeId: string },
  { state: RootState }
>("places/addPlaceWithBackend", async ({ placeId }, { getState, dispatch }) => {
  const placeState = getState().poi;
  const authState = getState().auth;
  const token = authState.session?.jwt_token;
  if (!token) throw new Error("Missing auth token");

  const place = placeState.places.find((p) => p.id === placeId);
  if (!place) throw new Error("Place not found");

  const imagesToUpload: LocalImage[] =
    placeState.localImages[placeId]?.filter((img) => !img.saved) || [];
  const formData = new FormData();

  formData.append("id", place.id);
  formData.append("name", place.name);
  formData.append("category", place.category);
  formData.append("location", place.location);
  if (place.note) formData.append("note", place.note);

  imagesToUpload.forEach((img) => {
    formData.append("image_id", img.id);
    const { filename, type: imageType } = getImageFileMeta(img.uri);
    formData.append("image", {
      uri: img.uri,
      name: filename,
      type: imageType,
    } as any);
  });

  const res = await fetch(`${API_BASE_URL}/places`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
    },
    body: formData,
  });

  if (!res.ok)
    throw new Error(`Failed to create place in backend: ${res.status} ${res.statusText}`);

  dispatch(markImagesSaved(placeId));
  const userId = getState().poi.userId;
  if (userId) {
    await dispatch(savePlacesAsync(userId));
  }
});

// Update a place in backend
export const updatePlaceWithBackend = createAsyncThunk<
  void,
  { placeId: string },
  { state: RootState }
>("places/updatePlaceWithBackend", async ({ placeId }, { getState, dispatch }) => {
  const placeState = getState().poi;
  const authState = getState().auth;
  const token = authState.session?.jwt_token;
  if (!token) throw new Error("Missing auth token");

  const place = placeState.places.find((p) => p.id === placeId);
  if (!place) throw new Error("Place not found");

  const imagesToUpload =
    placeState.localImages[placeId]?.filter((img) => !img.saved) || [];
  const imagesToDelete = placeState.deletedImages[placeId] || [];

  const formData = new FormData();
  formData.append("name", place.name);
  formData.append("category", place.category);
  formData.append("location", place.location);
  if (place.note) formData.append("note", place.note);

  imagesToUpload.forEach((img) => {
    formData.append("image_id", img.id);
    const { filename, type: imageType } = getImageFileMeta(img.uri);
    formData.append("image", {
      uri: img.uri,
      name: filename,
      type: imageType,
    } as any);
  });

  if (imagesToDelete.length > 0) {
    formData.append("delete_image_ids", JSON.stringify(imagesToDelete));
  }

  const res = await fetch(`${API_BASE_URL}/places/${place.id}`, {
    method: "PATCH",
    headers: {
      Authorization: `Bearer ${token}`,
    },
    body: formData,
  });

  if (!res.ok)
    throw new Error(`Failed to update place in backend: ${res.status} ${res.statusText}`);

  dispatch(markImagesSaved(placeId));
  const userId = getState().poi.userId;
  if (userId) {
    await dispatch(savePlacesAsync(userId));
  }
  // Clear deleted images after successful upload
  dispatch(clearDeletedImages(placeId));
});

// Delete a place in backend
export const deletePlaceWithBackend = createAsyncThunk<
  void,
  { placeId: string },
  { state: RootState }
>("places/deletePlaceWithBackend", async ({ placeId }, { getState, dispatch }) => {
  const state = getState();
  const token = state.auth.session?.jwt_token;

  if (!token) throw new Error("Missing auth token");

  const res = await fetch(`${API_BASE_URL}/places/${placeId}`, {
    method: "DELETE",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });

  if (!res.ok)
    throw new Error(`Failed to delete place in backend: ${res.status} ${res.statusText}`);
});
