#[cfg(test)]
mod test {
    use core_api::helper;
    use core_api::task::Task;
    use std::sync::Arc;
    use tokio::time;

    #[test]
    fn size() {
        println!("size of task is {} bytes", std::mem::size_of::<Task>());
    }

    #[test]
    fn exe_test() {
        let rt = helper::create_rt();
        let tsk = Task::new(
            0,
            "https://www.bilibili.com/video/BV1Ao4y1b7fj/?".to_owned(),
        );
        rt.block_on(tsk.execute()).unwrap();
    }

    #[test]
    fn switch_test() {
        let rt = helper::create_rt();
        let task = Arc::new(Task::new(
            0,
            "https://www.bilibili.com/video/BV1Ao4y1b7fj/?".to_owned(),
        ));
        rt.block_on(async move {
            let task_c = task.clone();
            tokio::spawn(async move {
                time::sleep(time::Duration::from_secs(1)).await;
                for _ in 0..50 {
                    task.switch();
                    time::sleep(time::Duration::from_millis(500)).await;
                }
            });
            task_c.execute().await.unwrap();
        });
    }

    #[test]
    fn cancel_test() {
        let rt = helper::create_rt();
        let task = Arc::new(Task::new(
            0,
            "https://www.bilibili.com/video/BV1Ao4y1b7fj/?".to_owned(),
        ));
        rt.block_on(async move {
            let task_c = task.clone();
            tokio::spawn(async move {
                time::sleep(time::Duration::from_secs(1)).await;
                for _ in 0..10 {
                    task.switch();
                    time::sleep(time::Duration::from_millis(50)).await;
                }
                task.cancel();
            });
            task_c.execute().await.unwrap();
            time::sleep(time::Duration::from_secs(2)).await;
        });
    }

    #[test]
    fn double_cancel_test() {
        let rt = helper::create_rt();
        let task = Arc::new(Task::new(
            0,
            "https://www.bilibili.com/video/BV1Ao4y1b7fj/?".to_owned(),
        ));
        rt.block_on(async move {
            let task_c = task.clone();
            tokio::spawn(async move {
                time::sleep(time::Duration::from_secs(1)).await;
                for _ in 0..10 {
                    time::sleep(time::Duration::from_millis(50)).await;
                    task.cancel();
                }
            });
            // no finish, the origin rx never drop, so can always send(x)
            // the true reason is task_c dropped, but task is not
            task_c.execute().await.unwrap();
            time::sleep(time::Duration::from_secs(2)).await;
        });
    }

    #[test]
    fn cancel_after_finished_test() {
        let rt = helper::create_rt();
        let task = Arc::new(Task::new(
            0,
            "https://www.bilibili.com/video/BV1Ao4y1b7fj/?".to_owned(),
        ));
        rt.block_on(async move {
            let task_c = task.clone();
            tokio::spawn(async move {
                time::sleep(time::Duration::from_secs(10)).await;
                task.cancel();
            });
            // due to Arc, task_c dropped but task is still holding the rx
            task_c.execute().await.unwrap();
        }); // task_c dropped here
        std::thread::sleep(std::time::Duration::from_secs(10))
    }

    #[test]
    fn get_process_test() {
        let rt = helper::create_rt();
        let task = Arc::new(Task::new(
            0,
            "https://www.bilibili.com/video/BV1Ao4y1b7fj/?".to_owned(),
        ));
        rt.block_on(async move {
            let task_c = task.clone();
            tokio::spawn(async move {
                loop {
                    println!("{}", task.process());
                    tokio::task::yield_now().await;
                }
            });
            // due to Arc, task_c dropped but task is still holding the rx
            task_c.execute().await.unwrap();
        }); // task_c dropped here
    }
}
