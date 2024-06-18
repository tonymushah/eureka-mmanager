# mangadex-desktop-api2

[![Rust][rust-action-badge]][rust-action]

`mangadex-desktop-api2` is a library for downloading and managing titles, covers, and chapters from [MangaDex][mangadex].

This is built on top of the [`mangadex-api`][mangadex-api].
But unlike the SDK, it allows you to download, read, update and delete chapter images, covers and titles metadata from your local device.

It might also get a package management system, sees [#174][pms]

## Some concepts that you need to know

This library now uses [Actor model][actor-model] to track the download process in real-time.

Each downloading task: chapter / cover images downloading and title metadata is an actor and which I call `Task`.
You can listen to a task to know it's state like if it's pending, loading, success, or error.
Each task have an unique id, corresponding to what it's downloading.
Downloading a manga will spawn a [`MangaDownloadTask`][manga-download-task] actor, cover will spawn a [`CoverDownloadTask`][cover-download-task], etc...

A Task is managed by a manager corresponding by the task type,
which means that there is a [`ChapterDownloadManager`][chapter-download-mananger], a [`CoverDownloadManager`][cover-download-manager] and a [`MangaDownloadManager`][manga-download-manager].

To manage all of this there is a top level [`DownloadManger`][download-manager] that allows you to interact with the manager.
But it also contain an [inner state][inner-state] allows to interact with the underlying [MangaDex API Client], [DirOptions API] and the [download history actor]

For example, you want to download a chapter like this [one][rascal-dnd-dreaming-girl-en-11] (a peak btw),

Anyway, if you're building a MangaDex desktop app (like [Special Eureka][special-eureka]) or a private downloading service, this library provide a bunch of features that might help you:

- Configurable download direc: W
- It's asynchronous: This library is built on top of [actix] actors (not [actix-web]) but it requires you to use [actix-rt] it

[rust-action-badge]: https://github.com/tonymushah/eureka-mmanager/actions/workflows/rust.yml/badge.svg
[rust-action]: https://github.com/tonymushah/eureka-mmanager/actions/workflows/rust.yml
[mangadex]: https://mangadex.org
[mangadex-api]: https://github.com/tonymushah/mangadex-api
[pms]: https://github.com/tonymushah/eureka-mmanager/issues/174
[special-eureka]: https://github.com/tonymushah/special-eureka
[actix]: https://docs.rs/actix/latest/actix/
[actix-web]: https://docs.rs/actix-web/latest/actix_web/
[actor-model]: https://en.wikipedia.org/wiki/Actor_model
[rascal-dnd-dreaming-girl-en-11]: https://mangadex.org/chapter/baf51491-4440-4ae4-8381-840804e99d32
[download-manager]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download.rs
[manga-download-manager]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/manga.rs
[cover-download-manager]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/covers.rs
[chapter-download-mananger]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/chapter.rs
[cover-download-task]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/cover/task.rs
[manga-download-task]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/manga/task.rs
[inner-state]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/state.rs
