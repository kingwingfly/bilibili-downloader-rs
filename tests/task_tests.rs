#[cfg(test)]
mod test {
    use core_api::helper;
    use core_api::task::Task;

    #[test]
    fn exe_test() {
        let rt = helper::create_rt();
        let tsk = Task::new(
            0,
            "https://www.bilibili.com/video/BV1Ao4y1b7fj/?".to_owned(),
            "../tests/video_downloads".to_owned(),
        );
        rt.block_on(tsk.execute()).unwrap();
    }
}
