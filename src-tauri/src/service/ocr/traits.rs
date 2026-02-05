pub trait OcrEngine: Send + Sync {
    /// Recognizes text from an image (encoded bytes).
    fn recognize(&self, image_bytes: &[u8]) -> anyhow::Result<String>;

    /// Returns the name of the engine.
    fn name(&self) -> &str;
}
