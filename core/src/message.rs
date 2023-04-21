use std::sync::Arc;

use crate::task::Task;

// state req
type StReq = (tokio::sync::oneshot::Sender<String>, usize);

#[derive(Debug)]
pub enum Message {
    Job(Arc<Task>),
    Process(StReq),
    Cancel(usize),
    Terminate,
}
