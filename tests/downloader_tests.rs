#[cfg(test)]
mod test {
    use core_api::downloader::Downloader;

    #[test]
    fn run_test() {
        let dl = Downloader::new();
        let target = "https://www.bilibili.com/video/BV1Ao4y1b7fj/?".to_owned();
        let save_dir = "../tests/video_downloads".to_owned();
        dl.run(target, save_dir);
        loop {}
    }
}
