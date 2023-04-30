#[cfg(test)]
mod test {
    use core_api::downloader::Downloader;

    #[test]
    fn run_test() {
        let dl = Downloader::new();
        let target = "https://www.bilibili.com/video/BV1Ao4y1b7fj/?".to_owned();
        let id = dl.add_task(target);
        loop {
            let process = dl.process(id);
            println!("{}", process);
            if dl.state(0) == 3 {
                break;
            }
        }
    }

    #[test]
    fn operate_when_no_task() {
        let dl = Downloader::new();
        dl.cancel(0);
        dl.switch_all();
        dl.switch(0);
    }
}
