import { API_BASE_URL } from "@/constants/env";
import { LocalImage } from "@/types/place";
import { createAsyncThunk } from "@reduxjs/toolkit";
import { RootState } from ".";
import { clearDeletedImages, clearLocalImages, deletePlace } from "./placeSlice";

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

  const imagesToUpload: LocalImage[] = placeState.localImages[placeId] || [];
  const formData = new FormData();

  formData.append("id", place.id);
  formData.append("name", place.name);
  formData.append("category", place.category);
  formData.append("location", place.location);
  if (place.note) formData.append("note", place.note);

  imagesToUpload.forEach((img) => {
    formData.append("image_id", img.id);
    formData.append("image", {
      uri: img.uri,
      name: `${img.id}.jpg`,
      type: "image/jpeg",
    } as any);
  });

  const res = await fetch(`${API_BASE_URL}/places`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "multipart/form-data",
    },
    body: formData,
  });

  if (!res.ok) throw new Error("Failed to create place in backend");

  // Clear local images after successful upload
  dispatch(clearLocalImages(placeId));
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

  const imagesToUpload = placeState.localImages[placeId] || [];
  const imagesToDelete = placeState.deletedImages[placeId] || [];

  const formData = new FormData();
  formData.append("name", place.name);
  formData.append("category", place.category);
  formData.append("location", place.location);
  if (place.note) formData.append("note", place.note);

  imagesToUpload.forEach((img) => {
    formData.append("image_id", img.id);
    formData.append("image", {
      uri: img.uri,
      name: `${img.id}.jpg`,
      type: "image/jpeg",
    } as any);
  });

  if (imagesToDelete.length > 0) {
    formData.append("delete_image_ids", JSON.stringify(imagesToDelete));
  }

  const res = await fetch(`${API_BASE_URL}/places/${place.id}`, {
    method: "PATCH",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "multipart/form-data",
    },
    body: formData,
  });

  if (!res.ok) throw new Error("Failed to update place in backend");

  // Clear local & deleted images after successful upload
  dispatch(clearLocalImages(placeId));
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

  if (!res.ok) throw new Error("Failed to delete place in backend");

  // Remove from state
  dispatch(deletePlace(placeId));
  dispatch(clearLocalImages(placeId));
  dispatch(clearDeletedImages(placeId));
});
