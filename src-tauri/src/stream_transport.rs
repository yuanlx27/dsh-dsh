//! Opaque physical carrier only. R owns logical mux frames and recovery.
use crate::{
    authentication::ServiceSession,
    bridge::Permit,
    errors::Failure,
    http_transport::{cancelled, permit_revoked},
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{Message, client::IntoClientRequest, protocol::WebSocketConfig},
};

// Pinned R's ws server retains ws's 100 MiB default maxPayload.
pub const MAX_FRAME_BYTES: usize = 100 * 1024 * 1024;
const QUEUE: usize = 1;
#[derive(Clone, Debug, PartialEq)]
pub enum Frame {
    Text(String),
    Binary(Vec<u8>),
}
impl Frame {
    fn len(&self) -> usize {
        match self {
            Self::Text(s) => s.len(),
            Self::Binary(b) => b.len(),
        }
    }
}
pub struct Downlink {
    pub sequence: u64,
    pub frame: Frame,
}
pub struct StreamTransport {
    session: Arc<ServiceSession>,
}
pub struct Stream {
    session: Arc<ServiceSession>,
    generation: u64,
    permit: Option<Permit>,
    cancel: watch::Receiver<bool>,
    stop: watch::Sender<bool>,
    sender: StreamSender,
    rx: mpsc::Receiver<Result<Frame, Failure>>,
    reader: JoinHandle<()>,
    writer: JoinHandle<()>,
    read_sequence: u64,
    failed: bool,
}
#[derive(Clone)]
pub struct StreamSender {
    tx: mpsc::Sender<Frame>,
    sequence: Arc<tokio::sync::Mutex<u64>>,
    stop: watch::Sender<bool>,
    cancel: watch::Receiver<bool>,
    session: Arc<ServiceSession>,
    generation: u64,
    permit: Option<Permit>,
}
impl StreamSender {
    pub async fn send(&self, sequence: u64, frame: Frame) -> Result<(), Failure> {
        let mut next = self.sequence.lock().await;
        if *self.stop.borrow()
            || check(
                &self.session,
                self.generation,
                &self.cancel,
                self.permit.as_ref(),
            )
            .is_err()
            || sequence != *next
            || frame.len() > MAX_FRAME_BYTES
        {
            self.stop.send_replace(true);
            return Err(Failure::Unavailable);
        }
        let result = tokio::select! { biased;
        _=cancelled(self.stop.subscribe())=>Err(Failure::Unavailable),
        _=cancelled(self.cancel.clone())=>Err(Failure::Unavailable),
        _=self.session.revoked(self.generation)=>Err(Failure::Unavailable),
        _=permit_revoked(self.permit.as_ref())=>Err(Failure::Unavailable),
        result=self.tx.send(frame)=>result.map_err(|_|Failure::Connection) };
        if result.is_err() {
            self.stop.send_replace(true);
        }
        result?;
        *next = next.checked_add(1).ok_or(Failure::Unavailable)?;
        Ok(())
    }
}
impl StreamTransport {
    pub fn new(session: Arc<ServiceSession>) -> Self {
        Self { session }
    }
    pub async fn connect_admitted(
        &self,
        permit: &Permit,
        cancel: watch::Receiver<bool>,
    ) -> Result<Stream, Failure> {
        if !permit.is_current() {
            return Err(Failure::Unavailable);
        }
        self.connect_owned(permit.runtime_generation(), cancel, Some(permit.clone()))
            .await
    }
    // Rust-only IO contracts, not a renderer command.
    pub async fn connect(
        &self,
        generation: u64,
        cancel: watch::Receiver<bool>,
    ) -> Result<Stream, Failure> {
        self.connect_owned(generation, cancel, None).await
    }
    async fn connect_owned(
        &self,
        generation: u64,
        cancel: watch::Receiver<bool>,
        permit: Option<Permit>,
    ) -> Result<Stream, Failure> {
        check(&self.session, generation, &cancel, permit.as_ref())?;
        let request = self
            .session
            .with_credentials(generation, |origin, cookie| {
                let mut target = origin
                    .join("/api/remote.mux")
                    .map_err(|_| Failure::Unavailable)?;
                target.set_scheme("ws").map_err(|_| Failure::Unavailable)?;
                let mut request = target
                    .as_str()
                    .into_client_request()
                    .map_err(|_| Failure::Unavailable)?;
                request.headers_mut().insert("cookie", cookie.clone());
                request.headers_mut().insert(
                    "origin",
                    origin
                        .origin()
                        .ascii_serialization()
                        .parse()
                        .map_err(|_| Failure::Unavailable)?,
                );
                Ok::<_, Failure>(request)
            })??;
        let config = WebSocketConfig::default()
            .max_message_size(Some(MAX_FRAME_BYTES))
            .max_frame_size(Some(MAX_FRAME_BYTES))
            .max_write_buffer_size(MAX_FRAME_BYTES + 128 * 1024);
        let (socket, _) = tokio::select! { biased;
            _=cancelled(cancel.clone())=>return Err(Failure::Unavailable),
            _=self.session.revoked(generation)=>return Err(Failure::Unavailable),
            _=permit_revoked(permit.as_ref())=>return Err(Failure::Unavailable),
            result=connect_async_with_config(request,Some(config),true)=>result.map_err(|_|Failure::Connection)?,
        };
        check(&self.session, generation, &cancel, permit.as_ref())?;
        let (mut sink, mut source) = socket.split();
        let (stop, signal) = watch::channel(false);
        let (tx, mut outgoing) = mpsc::channel::<Frame>(QUEUE);
        let (incoming, rx) = mpsc::channel(QUEUE);
        let session = self.session.clone();
        let p = permit.clone();
        let c = cancel.clone();
        let s = signal.clone();
        let writer_stop = stop.clone();
        let writer = tokio::spawn(async move {
            loop {
                tokio::select! { biased;
                    _=cancelled(c.clone())=>break, _=cancelled(s.clone())=>break,
                    _=session.revoked(generation)=>break, _=permit_revoked(p.as_ref())=>break,
                    frame=outgoing.recv()=>{ let Some(frame)=frame else { break; };
                        let frame=match frame { Frame::Text(t)=>Message::Text(t.into()),Frame::Binary(b)=>Message::Binary(b.into()) };
                        // Select cancellation even while the peer stops reading.
                        tokio::select! { biased; _=cancelled(c.clone())=>break, _=cancelled(s.clone())=>break,
                            _=session.revoked(generation)=>break,_=permit_revoked(p.as_ref())=>break,
                            result=sink.send(frame)=>if result.is_err(){break;} }
                    }
                }
            }
            writer_stop.send_replace(true);
        });
        let session = self.session.clone();
        let p = permit.clone();
        let c = cancel.clone();
        let s = signal;
        let reader = tokio::spawn(async move {
            loop {
                let item = tokio::select! { biased; _=cancelled(c.clone())=>break,_=cancelled(s.clone())=>break,
                _=session.revoked(generation)=>break,_=permit_revoked(p.as_ref())=>break,
                item=source.next()=>item };
                let frame = match item {
                    Some(Ok(Message::Text(t))) => Ok(Frame::Text(t.to_string())),
                    Some(Ok(Message::Binary(b))) => Ok(Frame::Binary(b.to_vec())),
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => continue,
                    Some(Err(_)) => Err(Failure::Connection),
                };
                let terminal = frame.is_err();
                tokio::select! { biased; _=cancelled(c.clone())=>break,_=cancelled(s.clone())=>break,
                _=session.revoked(generation)=>break,_=permit_revoked(p.as_ref())=>break,
                result=incoming.send(frame)=>if result.is_err(){break;} }
                if terminal {
                    break;
                }
            }
        });
        let sender = StreamSender {
            tx,
            sequence: Arc::new(tokio::sync::Mutex::new(0)),
            stop: stop.clone(),
            cancel: cancel.clone(),
            session: self.session.clone(),
            generation,
            permit: permit.clone(),
        };
        Ok(Stream {
            session: self.session.clone(),
            generation,
            permit,
            cancel,
            stop,
            sender,
            rx,
            reader,
            writer,
            read_sequence: 0,
            failed: false,
        })
    }
}
impl Stream {
    fn check(&mut self) -> Result<(), Failure> {
        if self.failed
            || *self.stop.borrow()
            || check(
                &self.session,
                self.generation,
                &self.cancel,
                self.permit.as_ref(),
            )
            .is_err()
        {
            self.failed = true;
            self.stop.send_replace(true);
            return Err(Failure::Unavailable);
        }
        Ok(())
    }
    pub fn sender(&self) -> StreamSender {
        self.sender.clone()
    }
    pub async fn send(&mut self, sequence: u64, frame: Frame) -> Result<(), Failure> {
        self.sender.send(sequence, frame).await
    }
    pub async fn next_frame(&mut self) -> Result<Option<Downlink>, Failure> {
        self.check()?;
        let item = tokio::select! { biased; _=cancelled(self.stop.subscribe())=>return Err(Failure::Unavailable), _=cancelled(self.cancel.clone())=>return Err(Failure::Unavailable),
        _=self.session.revoked(self.generation)=>return Err(Failure::Unavailable),_=permit_revoked(self.permit.as_ref())=>return Err(Failure::Unavailable),
        item=self.rx.recv()=>item };
        self.check()?;
        match item {
            Some(Ok(frame)) => {
                let sequence = self.read_sequence;
                self.read_sequence += 1;
                Ok(Some(Downlink { sequence, frame }))
            }
            Some(Err(e)) => {
                self.failed = true;
                self.stop.send_replace(true);
                Err(e)
            }
            None => {
                self.stop.send_replace(true);
                Ok(None)
            }
        }
    }
}
impl Drop for Stream {
    fn drop(&mut self) {
        self.stop.send_replace(true);
        self.reader.abort();
        self.writer.abort();
    }
}
fn check(
    session: &ServiceSession,
    generation: u64,
    cancel: &watch::Receiver<bool>,
    permit: Option<&Permit>,
) -> Result<(), Failure> {
    if *cancel.borrow()
        || cancel.has_changed().is_err()
        || !session.is_current(generation)
        || permit.is_some_and(|p| !p.is_current())
    {
        return Err(Failure::Unavailable);
    }
    Ok(())
}
