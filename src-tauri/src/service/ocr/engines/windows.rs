use crate::service::ocr::traits::OcrEngine;
use windows::Graphics::Imaging::BitmapDecoder;
use windows::Media::Ocr::OcrEngine as WinOcr;
use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};

pub struct WindowsNativeOcr;

impl OcrEngine for WindowsNativeOcr {
    fn recognize(&self, image_bytes: &[u8]) -> anyhow::Result<String> {
        // 1. Create a stream from bytes
        let stream = InMemoryRandomAccessStream::new()?;
        let writer = DataWriter::CreateDataWriter(&stream)?;
        writer.WriteBytes(image_bytes)?;
        writer.StoreAsync()?.get()?;
        writer.FlushAsync()?.get()?;
        stream.Seek(0)?;

        // 2. Decode the image to SoftwareBitmap
        let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
        let bitmap = decoder.GetSoftwareBitmapAsync()?.get()?;

        // 3. Perform OCR
        let engine = WinOcr::TryCreateFromUserProfileLanguages()?;
        let result = engine.RecognizeAsync(&bitmap)?.get()?;

        Ok(result.Text()?.to_string())
    }

    fn name(&self) -> &str {
        "Windows Native"
    }
}
