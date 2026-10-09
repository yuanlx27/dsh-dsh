//! Bounded native handle/action store. Entry always requires an admitted Permit.
use crate::{
    authentication::ServiceSession,
    bridge::{Operation, Permit},
    errors::Failure,
    http_transport::{self, Body, HttpRequest, Transport},
    stream_transport::{self, Frame, StreamSender, StreamTransport},
};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use reqwest::{
    Method,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::Read,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::{Mutex as AsyncMutex, watch};
pub type Session = Arc<dyn Fn(&Permit) -> Result<Arc<ServiceSession>, Failure> + Send + Sync>;
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
enum HttpAction {
    Begin {
        path: String,
        method: String,
        headers: Vec<(String, String)>,
    },
    Write {
        handle: String,
        sequence: u64,
        data: String,
    },
    Finish {
        handle: String,
    },
    Read {
        handle: String,
    },
    Cancel {
        handle: String,
    },
}
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
enum StreamAction {
    Begin {},
    Write {
        handle: String,
        sequence: u64,
        chunk: u64,
        data: String,
        end: bool,
    },
    Read {
        handle: String,
    },
    Cancel {
        handle: String,
    },
}
struct HttpState {
    input: Option<HttpRequest>,
    reply: Option<http_transport::Reply>,
}
struct SocketRead {
    stream: stream_transport::Stream,
    pending: Vec<u8>,
    offset: usize,
    sequence: u64,
    kind: &'static str,
}
struct SocketWrite {
    sender: StreamSender,
    sequence: u64,
    chunk: u64,
    frame: Vec<u8>,
}
struct Socket {
    read: AsyncMutex<SocketRead>,
    write: AsyncMutex<SocketWrite>,
}
enum Io {
    Http(AsyncMutex<HttpState>),
    Stream(Socket),
}
struct Job {
    scope: Permit,
    cancel: watch::Sender<bool>,
    io: Io,
    reserved: AtomicUsize,
    budget: Arc<AtomicUsize>,
}
impl Job {
    fn release(&self) {
        let n = self.reserved.swap(0, Ordering::SeqCst);
        self.budget.fetch_sub(n, Ordering::SeqCst);
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.send_replace(true);
        self.release();
    }
}
pub struct Dispatcher {
    jobs: Mutex<HashMap<String, Arc<Job>>>,
    budget: Arc<AtomicUsize>,
    session: Session,
    opening: AsyncMutex<()>,
}
impl Dispatcher {
    pub fn new(session: Session) -> Arc<Self> {
        Arc::new(Self {
            jobs: Mutex::new(HashMap::new()),
            budget: Arc::new(AtomicUsize::new(0)),
            session,
            opening: AsyncMutex::new(()),
        })
    }
    fn insert(
        self: &Arc<Self>,
        scope: Permit,
        cancel: watch::Sender<bool>,
        io: Io,
    ) -> Result<String, Failure> {
        let mut jobs = self.jobs.lock().map_err(|_| Failure::Unavailable)?;
        if jobs.len() >= 32 || !scope.is_current() {
            return Err(Failure::Unavailable);
        }
        let mut bytes = [0u8; 24];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut file| file.read_exact(&mut bytes))
            .map_err(|_| Failure::Unavailable)?;
        let id = URL_SAFE_NO_PAD.encode(bytes);
        let done = cancel.subscribe();
        jobs.insert(
            id.clone(),
            Arc::new(Job {
                scope: scope.clone(),
                cancel,
                io,
                reserved: AtomicUsize::new(0),
                budget: self.budget.clone(),
            }),
        );
        drop(jobs);
        let weak = Arc::downgrade(self);
        let key = id.clone();
        tokio::spawn(async move {
            tokio::select! {_=scope.revoked()=>{},_=http_transport::cancelled(done)=>{}}
            if let Some(store) = weak.upgrade() {
                store.remove(&key);
            }
        });
        Ok(id)
    }
    fn remove(&self, id: &str) {
        if let Ok(mut jobs) = self.jobs.lock()
            && let Some(job) = jobs.remove(id)
        {
            job.cancel.send_replace(true);
        }
    }
    fn job(&self, scope: &Permit, id: &str) -> Result<Arc<Job>, Failure> {
        let job = self
            .jobs
            .lock()
            .map_err(|_| Failure::Unavailable)?
            .get(id)
            .cloned()
            .ok_or(Failure::Unavailable)?;
        if !scope.is_current() || !job.scope.same_scope(scope) || *job.cancel.borrow() {
            return Err(Failure::Unavailable);
        }
        Ok(job)
    }
    fn cancel(&self, scope: &Permit, id: &str) -> Result<Value, Failure> {
        if let Ok(jobs) = self.jobs.lock()
            && let Some(job) = jobs.get(id)
            && !job.scope.same_scope(scope)
        {
            return Err(Failure::Unavailable);
        }
        self.remove(id);
        Ok(Value::Null)
    }
    pub async fn call(
        self: &Arc<Self>,
        scope: Permit,
        operation: Operation,
        payload: Value,
    ) -> Result<Value, Failure> {
        if !scope.is_current() {
            return Err(Failure::Unavailable);
        }
        let session = (self.session)(&scope)?;
        match operation {
            Operation::Http => {
                self.http(
                    scope,
                    session,
                    serde_json::from_value(payload).map_err(|_| Failure::Unavailable)?,
                )
                .await
            }
            Operation::Stream => {
                self.stream(
                    scope,
                    session,
                    serde_json::from_value(payload).map_err(|_| Failure::Unavailable)?,
                )
                .await
            }
            Operation::Bootstrap => Err(Failure::Unavailable),
        }
    }
    async fn http(
        self: &Arc<Self>,
        scope: Permit,
        session: Arc<ServiceSession>,
        action: HttpAction,
    ) -> Result<Value, Failure> {
        match action {
            HttpAction::Begin {
                path,
                method,
                headers,
            } => {
                http_transport::validate_target(&path)?;
                let method =
                    Method::from_bytes(method.as_bytes()).map_err(|_| Failure::Unavailable)?;
                if !matches!(
                    method,
                    Method::GET
                        | Method::HEAD
                        | Method::POST
                        | Method::PUT
                        | Method::PATCH
                        | Method::DELETE
                        | Method::OPTIONS
                ) || headers.len() > 256
                {
                    return Err(Failure::Unavailable);
                }
                let mut map = HeaderMap::new();
                for (name, value) in headers {
                    map.append(
                        HeaderName::from_bytes(name.as_bytes())
                            .map_err(|_| Failure::Unavailable)?,
                        HeaderValue::from_str(&value).map_err(|_| Failure::Unavailable)?,
                    );
                }
                let (cancel, _) = watch::channel(false);
                let handle = self.insert(
                    scope,
                    cancel,
                    Io::Http(AsyncMutex::new(HttpState {
                        input: Some(HttpRequest {
                            path,
                            method,
                            headers: map,
                            body: Body::default(),
                        }),
                        reply: None,
                    })),
                )?;
                Ok(json!({"handle":handle}))
            }
            HttpAction::Cancel { handle } => self.cancel(&scope, &handle),
            HttpAction::Write {
                handle,
                sequence,
                data,
            } => {
                let job = self.job(&scope, &handle)?;
                let Io::Http(state) = &job.io else {
                    return Err(Failure::Unavailable);
                };
                let mut state = state.lock().await;
                let result = (|| {
                    let bytes = decode(&data)?;
                    let n = bytes.len();
                    self.budget
                        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                            current
                                .checked_add(n)
                                .filter(|v| *v <= http_transport::MAX_BODY_BYTES)
                        })
                        .map_err(|_| Failure::Unavailable)?;
                    job.reserved.fetch_add(n, Ordering::SeqCst);
                    state
                        .input
                        .as_mut()
                        .ok_or(Failure::Unavailable)?
                        .body
                        .push(sequence, bytes)?;
                    Ok(Value::Null)
                })();
                if result.is_err() {
                    self.remove(&handle);
                }
                result
            }
            HttpAction::Finish { handle } => {
                let job = self.job(&scope, &handle)?;
                let Io::Http(state) = &job.io else {
                    return Err(Failure::Unavailable);
                };
                let input = state
                    .lock()
                    .await
                    .input
                    .take()
                    .ok_or(Failure::Unavailable)?;
                let result = Transport::new(session)
                    .fetch_admitted(&scope, input, job.cancel.subscribe())
                    .await;
                job.release();
                let reply = match result {
                    Ok(r) => r,
                    Err(e) => {
                        self.remove(&handle);
                        return Err(e);
                    }
                };
                let headers = reply
                    .headers
                    .iter()
                    .map(|(k, v)| Ok((k.as_str(), v.to_str().map_err(|_| Failure::Connection)?)))
                    .collect::<Result<Vec<_>, Failure>>();
                let headers = match headers {
                    Ok(headers) => headers,
                    Err(error) => {
                        self.remove(&handle);
                        return Err(error);
                    }
                };
                let response = json!({"status":reply.status,"headers":headers});
                state.lock().await.reply = Some(reply);
                Ok(response)
            }
            HttpAction::Read { handle } => {
                let job = self.job(&scope, &handle)?;
                let Io::Http(state) = &job.io else {
                    return Err(Failure::Unavailable);
                };
                let mut state = state.lock().await;
                let result = state
                    .reply
                    .as_mut()
                    .ok_or(Failure::Unavailable)?
                    .next_chunk()
                    .await;
                match result {
                    Ok(Some(chunk)) => {
                        Ok(json!({"sequence":chunk.sequence,"data":STANDARD.encode(chunk.bytes)}))
                    }
                    Ok(None) => {
                        self.remove(&handle);
                        Ok(Value::Null)
                    }
                    Err(e) => {
                        self.remove(&handle);
                        Err(e)
                    }
                }
            }
        }
    }
    async fn stream(
        self: &Arc<Self>,
        scope: Permit,
        session: Arc<ServiceSession>,
        action: StreamAction,
    ) -> Result<Value, Failure> {
        match action {
            StreamAction::Begin {} => {
                let _opening = self.opening.lock().await;
                {
                    let jobs = self.jobs.lock().map_err(|_| Failure::Unavailable)?;
                    if jobs
                        .values()
                        .any(|job| job.scope.same_scope(&scope) && matches!(job.io, Io::Stream(_)))
                    {
                        return Err(Failure::Unavailable);
                    }
                }
                let (cancel, signal) = watch::channel(false);
                let stream = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    StreamTransport::new(session).connect_admitted(&scope, signal),
                )
                .await
                .map_err(|_| Failure::Connection)??;
                let write = SocketWrite {
                    sender: stream.sender(),
                    sequence: 0,
                    chunk: 0,
                    frame: vec![],
                };
                let read = SocketRead {
                    stream,
                    pending: vec![],
                    offset: 0,
                    sequence: 0,
                    kind: "text",
                };
                let handle = self.insert(
                    scope,
                    cancel,
                    Io::Stream(Socket {
                        read: AsyncMutex::new(read),
                        write: AsyncMutex::new(write),
                    }),
                )?;
                Ok(json!({"handle":handle}))
            }
            StreamAction::Cancel { handle } => self.cancel(&scope, &handle),
            StreamAction::Write {
                handle,
                sequence,
                chunk,
                data,
                end,
            } => {
                let job = self.job(&scope, &handle)?;
                let Io::Stream(socket) = &job.io else {
                    return Err(Failure::Unavailable);
                };
                let mut write = socket.write.lock().await;
                let bytes = match decode(&data) {
                    Ok(b) => b,
                    Err(e) => {
                        self.remove(&handle);
                        return Err(e);
                    }
                };
                if sequence != write.sequence
                    || chunk != write.chunk
                    || (!end && bytes.len() != http_transport::MAX_CHUNK_BYTES)
                    || bytes.len() > stream_transport::MAX_FRAME_BYTES - write.frame.len()
                {
                    self.remove(&handle);
                    return Err(Failure::Unavailable);
                }
                write.frame.extend(bytes);
                write.chunk += 1;
                if end {
                    let text = match String::from_utf8(std::mem::take(&mut write.frame)) {
                        Ok(t) => t,
                        Err(_) => {
                            self.remove(&handle);
                            return Err(Failure::Unavailable);
                        }
                    };
                    if let Err(e) = write.sender.send(sequence, Frame::Text(text)).await {
                        self.remove(&handle);
                        return Err(e);
                    }
                    write.sequence += 1;
                    write.chunk = 0;
                }
                Ok(Value::Null)
            }
            StreamAction::Read { handle } => {
                let job = self.job(&scope, &handle)?;
                let Io::Stream(socket) = &job.io else {
                    return Err(Failure::Unavailable);
                };
                let mut read = socket.read.lock().await;
                if read.offset == read.pending.len() {
                    let frame = match read.stream.next_frame().await {
                        Ok(Some(f)) => f,
                        Ok(None) => {
                            self.remove(&handle);
                            return Ok(Value::Null);
                        }
                        Err(e) => {
                            self.remove(&handle);
                            return Err(e);
                        }
                    };
                    read.sequence = frame.sequence;
                    let (kind, bytes) = match frame.frame {
                        Frame::Text(t) => ("text", t.into_bytes()),
                        Frame::Binary(b) => ("binary", b),
                    };
                    read.kind = kind;
                    read.pending = bytes;
                    read.offset = 0;
                }
                let end = (read.offset + http_transport::MAX_CHUNK_BYTES).min(read.pending.len());
                let response = json!({"sequence":read.sequence,"kind":read.kind,"data":STANDARD.encode(&read.pending[read.offset..end]),"end":end==read.pending.len()});
                read.offset = end;
                Ok(response)
            }
        }
    }
}
fn decode(input: &str) -> Result<Vec<u8>, Failure> {
    if input.len() > 87384 {
        return Err(Failure::Unavailable);
    }
    let bytes = STANDARD.decode(input).map_err(|_| Failure::Unavailable)?;
    if bytes.len() > http_transport::MAX_CHUNK_BYTES {
        return Err(Failure::Unavailable);
    }
    Ok(bytes)
}
