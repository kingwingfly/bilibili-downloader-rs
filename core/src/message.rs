//! The messages that would be send in channels

use crate::task::Task;

// state req
type PrcReq = (tokio::sync::oneshot::Sender<String>, usize);
type TtReq = (tokio::sync::oneshot::Sender<String>, usize);
type StReq = (tokio::sync::oneshot::Sender<usize>, usize);

#[derive(Debug)]
pub enum Message {
    Job(Task),
    Process(PrcReq),
    State(StReq),
    Title(TtReq),
    Cancel(usize),
    Switch(usize),
    SwitchAll,
    Terminate,
}
