use std::fs::{ReadDir, DirEntry};

use futures::Stream;

struct ReadDirStream{
    read_dir : ReadDir
}

impl Stream for ReadDirStream {
    type Item = std::io::Result<DirEntry>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.read_dir.count()))
    }

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        while let Some(entry) = self.read_dir.next() {
            return std::task::Poll::Ready(Some(entry))
        }
        std::task::Poll::Ready(None)
    }
}