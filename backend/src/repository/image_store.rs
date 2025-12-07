use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct ImageStore {
    base_dir: Arc<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ImageUpload {
    pub id: Uuid,
    pub file_name: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StoredImage {
    pub id: Uuid,
    pub file_name: String,
}

impl ImageStore {
    pub fn new(base_dir: PathBuf) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(&base_dir)?;
        Ok(Self {
            base_dir: Arc::new(base_dir),
        })
    }

    pub fn path_for(&self, place_id: Uuid, file_name: &str) -> PathBuf {
        self.base_dir.join(place_id.to_string()).join(file_name)
    }

    pub async fn save_images(
        &self,
        place_id: Uuid,
        uploads: Vec<ImageUpload>,
    ) -> Result<Vec<StoredImage>, std::io::Error> {
        let mut stored = Vec::new();
        for upload in uploads {
            let file_name = self
                .write_image(
                    place_id,
                    upload.id,
                    upload.file_name.as_deref(),
                    &upload.bytes,
                )
                .await?;
            stored.push(StoredImage {
                id: upload.id,
                file_name,
            });
        }
        Ok(stored)
    }

    pub async fn cleanup_images(&self, place_id: Uuid, stored: &[StoredImage]) {
        for image in stored {
            let path = self.path_for(place_id, &image.file_name);
            if let Err(err) = fs::remove_file(&path).await {
                error!(?err, ?path, "failed to cleanup image file after error");
            }
        }
    }

    pub async fn remove_files(&self, place_id: Uuid, file_names: &[String]) {
        for file_name in file_names {
            let path = self.path_for(place_id, file_name);
            if let Err(err) = fs::remove_file(&path).await {
                error!(?err, ?path, "failed to delete image file");
            }
        }
    }

    pub async fn remove_place_dir(&self, place_id: Uuid) {
        let dir = self.base_dir.join(place_id.to_string());
        if let Err(err) = fs::remove_dir_all(&dir).await {
            if err.kind() != std::io::ErrorKind::NotFound {
                error!(?err, ?dir, "failed to delete place image directory");
            }
        }
    }

    pub async fn get_image(
        &self,
        place_id: Uuid,
        file_name: &str,
    ) -> Result<Vec<u8>, std::io::Error> {
        let path = self.path_for(place_id, file_name);
        fs::read(path).await
    }

    async fn write_image(
        &self,
        place_id: Uuid,
        image_id: Uuid,
        file_name: Option<&str>,
        bytes: &[u8],
    ) -> Result<String, std::io::Error> {
        let place_dir = self.base_dir.join(place_id.to_string());
        fs::create_dir_all(&place_dir).await?;

        let extension = file_name
            .and_then(|name| Path::new(name).extension())
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{}", ext));

        let stored_file_name = format!(
            "{}{}",
            image_id,
            extension.unwrap_or_else(|| String::from(""))
        );

        let full_path = place_dir.join(&stored_file_name);
        fs::write(full_path, bytes).await?;

        Ok(stored_file_name)
    }
}
