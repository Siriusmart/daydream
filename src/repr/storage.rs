use std::{
    error::Error,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

#[async_trait]
pub trait Storage: Send + Sync {
    async fn read(
        &self,
        dir: &str,
        name: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error + Send + Sync>>;
    async fn write(
        &self,
        dir: &str,
        name: &str,
        content: &[u8],
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub struct FsStorage {
    root: PathBuf,
}

impl FsStorage {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[async_trait]
impl Storage for FsStorage {
    async fn read(
        &self,
        dir: &str,
        name: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error + Send + Sync>> {
        use tokio::fs;
        let path = self.root.join(dir).join(name);
        if fs::try_exists(&path).await? {
            Ok(Some(fs::read(path).await?))
        } else {
            Ok(None)
        }
    }

    async fn write(
        &self,
        dir: &str,
        name: &str,
        content: &[u8],
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        use tokio::fs;
        fs::create_dir_all(self.root.join(dir)).await?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.root.join(dir).join(name))
            .await?;

        Ok(file.write_all(content).await?)
    }
}
