use crate::task::Task;

// state req
type PrcReq = (tokio::sync::oneshot::Sender<String>, usize);

#[derive(Debug)]
pub enum Message {
    Job(Task),
    Process(PrcReq),
    Cancel(usize),
    Switch(usize),
    SwitchAll,
    Terminate,
}
