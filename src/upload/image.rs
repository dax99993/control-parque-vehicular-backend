use image::io::Reader;
use std::io::Cursor;
use image::DynamicImage;
use image::imageops::FilterType;

use crate::error::error_chain_fmt;


#[derive(thiserror::Error)]
pub enum ImageError {
    #[error("cannot guess image format from bytes")]
    FormatError,
    #[error("invalid image bytes")]
    DecodeError(#[from] anyhow::Error),
    #[error("cannot save image")]
    SaveError(#[source] anyhow::Error),
}

impl std::fmt::Debug for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[tracing::instrument(
    name = "Save image from bytes",
    skip(image_bytes)
)]
pub async fn save_image(
    image_bytes: Vec<u8>,
    save_path: &str,
    resize: Option<(u32,u32)>,
) -> Result<(), ImageError> {
    let reader = Reader::new(Cursor::new(image_bytes))
        .with_guessed_format()
        .map_err(|_| ImageError::FormatError)?;

    let image: DynamicImage = reader.decode()
        .map_err(|e| ImageError::DecodeError(e.into()))?;

    let image = if resize.is_some() {
        let (width, height) = resize.unwrap();
        image.resize_exact(width, height, FilterType::Nearest)
    } else {
        image
    };

    image.save(save_path)
        .map_err(|e| ImageError::SaveError(e.into()))
}

use actix_web::{web, HttpRequest, http::header::CONTENT_LENGTH};
use actix_multipart::Multipart;
use futures::TryStreamExt as _;
use mime::{Mime, IMAGE_PNG, IMAGE_JPEG, IMAGE_GIF};

#[tracing::instrument(
    name = "Handle single image uploading from multipart",
    skip(payload, req)
)]
pub async fn handle_picture_multipart(
    mut payload: Multipart,
    req: HttpRequest,
    save_path: &str,
    resize: Option<(u32,u32)>,
) -> Result<(), anyhow::Error> {

    let content_length: usize = match req.headers().get(CONTENT_LENGTH) {
        Some(header_value) => header_value.to_str().unwrap_or("0").parse().unwrap(),
        None => "0".parse().unwrap(),
    } ;

    let max_file_size: usize = 1024 * 1024 * 10; // 10 Mb file
    let max_file_count: usize = 1;
    let legal_filetypes: [Mime; 3] = [IMAGE_GIF, IMAGE_PNG, IMAGE_JPEG];
    let mut current_count: usize = 0;
    let mut image_bytes: Vec<u8> = vec![]; 

    if save_path.is_empty() || !save_path.contains(".jpeg") { return Err(anyhow::anyhow!("Invalid save path")) };
    dbg!(&content_length);
    dbg!(&max_file_size);
    if content_length > max_file_size { return Err(anyhow::anyhow!("Bad request")) };

    loop {
        if current_count == max_file_count { break; }
        if let Ok(Some(mut field)) = payload.try_next().await {
            let filetype: Option<&Mime> = field.content_type();
            if filetype.is_none() { continue; }
            if !legal_filetypes.contains(&filetype.unwrap()) { continue; }
            
            while let Ok(Some(chunk)) = field.try_next().await {
                image_bytes.extend_from_slice(&chunk);
            }

        } else { break; }
        current_count += 1;
    }
    // No image received
    if image_bytes.is_empty() {
        return Err(anyhow::anyhow!("Bad request"));
    } else {
        let save_path = format!("{}", save_path);
        web::block(move || async move {
            save_image(image_bytes, &save_path, resize).await
        })
        .await
        .map_err(|_| anyhow::anyhow!("Couldnt create threadpool"))?
        .await
        .map_err(|_| anyhow::anyhow!("Couldnt save image"))?;
    }

    Ok(())
}
