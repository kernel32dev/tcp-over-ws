pub mod addr;

use std::{
    collections::HashMap,
    io::ErrorKind,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use async_tungstenite::{
    tungstenite::{client::IntoClientRequest, http, Message, Utf8Bytes},
    WebSocketStream,
};
use either::Either::{Left, Right};
use futures::{AsyncRead, AsyncWrite, StreamExt};
use tokio_util::bytes::Bytes;

pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const MAX_TIMEOUT_MS: u64 = 300_000;
const MAX_BYTES_WS_MESSAGE: usize = 1024 * 8;

pub struct Session {
    tcp: Option<tokio::net::TcpStream>,
    id: u64,
    timeout: u64,
    /// the amount of bytes confirmed to have been written to the tcp stream
    write_cursor: u64,
    /// the amount of bytes confirmed to have been received by the websocket client
    read_cursor: u64,
    /// buffer of possibly unreceived bytes, in the array of all bytes returned by the tcp stream these bytes start at `read_cursor`
    buffer: Vec<u8>,
    closed: bool,
    last_use: Instant,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    WsToTcp,
    TcpToWs,
}
impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::WsToTcp => f.write_str("WsToTcp"),
            Direction::TcpToWs => f.write_str("TcpToWs"),
        }
    }
}

const UNKNOWN_ID: &'static str = match usize::BITS {
    64 => "????????????????",
    32 => "????????",
    _ => "?",
};

#[tokio::main]
pub async fn tcp_to_ws_service(
    connect_request: http::Request<()>,
    listen: Vec<SocketAddr>,
    timeout: u64,
) -> std::io::Result<std::convert::Infallible> {
    let server = tokio::net::TcpListener::bind(&listen[..]).await?;
    loop {
        match server.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_tcp_to_ws_connection(
                    connect_request.clone(),
                    stream,
                    timeout,
                ));
            }
            Err(error) => {
                println!("Aviso: erro ao tentar aceitar conecção: {error:?}");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn handle_tcp_to_ws_connection(
    mut connect_request: http::Request<()>,
    stream: tokio::net::TcpStream,
    timeout: u64,
) {
    let dir = Direction::TcpToWs;
    let mut id = 0;
    while id == 0 {
        id = rand::random();
    }

    println!("[{dir} {id:016x}] Nova conecção tcp");
    connect_request.headers_mut().insert(
        http::HeaderName::from_static("x-tow-id"),
        http::HeaderValue::from_maybe_shared(id.to_string()).unwrap(),
    );
    connect_request.headers_mut().insert(
        http::HeaderName::from_static("x-tow-timeout"),
        http::HeaderValue::from_maybe_shared(timeout.to_string()).unwrap(),
    );

    let mut session = Session {
        tcp: Some(stream),
        id,
        timeout,
        write_cursor: 0,
        read_cursor: 0,
        buffer: Vec::with_capacity(1024 * 4),
        closed: false,
        last_use: Instant::now(),
    };

    let mut last_connect = Instant::now();

    loop {
        let timeout = last_connect.elapsed() > Duration::from_millis(timeout);
        match async_tungstenite::tokio::connect_async(connect_request.clone()).await {
            Ok((websocket, _)) => {
                println!("[{dir} {id:016x}] Websocket adquirido");
                handle_live_session(Direction::TcpToWs, &mut session, websocket).await;
                println!("[{dir} {id:016x}] Websocket pertido");
                if session.closed {
                    println!("[{dir} {id:016x}] Encerrado");
                    return;
                }
                last_connect = Instant::now();
            }
            Err(error) => {
                if timeout {
                    println!("Erro: erro em nova conecção do ws: {error:?} (timeout)");
                    return;
                }
                println!("Aviso: erro em nova conecção do ws: {error:?}");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

#[tokio::main]
pub async fn ws_to_tcp_service(
    connect_addr: SocketAddr,
    listen: Vec<SocketAddr>,
) -> std::io::Result<std::convert::Infallible> {
    let server = tokio::net::TcpListener::bind(&listen[..]).await?;
    let sessions = &*Box::leak(Box::new(tokio::sync::RwLock::new(HashMap::<
        u64,
        Arc<tokio::sync::Mutex<Session>>,
    >::new())));
    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let mut lock = sessions.write().await;
            let ids = lock
                .values()
                .filter_map(|x| {
                    x.clone()
                        .try_lock_owned()
                        .ok()
                        .filter(|x| x.last_use.elapsed() > Duration::from_millis(x.timeout))
                        .map(|x| x.id)
                })
                .collect::<Vec<_>>();
            for id in &ids {
                lock.remove(id);
            }
        }
    });
    loop {
        match server.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_ws_to_tcp_connection(sessions, connect_addr, stream));
            }
            Err(error) => {
                println!("Aviso: erro ao tentar aceitar conecção: {error:?}");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn handle_ws_to_tcp_connection(
    sessions: &'static tokio::sync::RwLock<HashMap<u64, Arc<tokio::sync::Mutex<Session>>>>,
    connect_addr: SocketAddr,
    stream: tokio::net::TcpStream,
) {
    let dir = Direction::WsToTcp;
    println!("[{dir} {UNKNOWN_ID}] Nova conecção tcp");
    let mut tow_id = 0;
    let mut tow_timeout = 0;
    let result = async_tungstenite::tokio::accept_hdr_async(
        stream,
        |req: &http::Request<()>, res: http::Response<()>| {
            tow_id = req
                .headers()
                .get(http::HeaderName::from_static("x-tow-id"))
                .and_then(|x| x.to_str().ok().and_then(|x| x.parse::<u64>().ok()))
                .unwrap_or(0);
            tow_timeout = req
                .headers()
                .get(http::HeaderName::from_static("x-tow-timeout"))
                .and_then(|x| x.to_str().ok().and_then(|x| x.parse::<u64>().ok()))
                .unwrap_or(DEFAULT_TIMEOUT_MS)
                .min(MAX_TIMEOUT_MS);
            Ok(res)
        },
    )
    .await;
    match result {
        Ok(websocket) => {
            println!("[{dir} {tow_id:016x}] Websocket adquirido");
            let session = sessions.read().await.get(&tow_id).cloned();
            let session = match session {
                Some(session) => session,
                None => {
                    let mut lock = sessions.write().await;
                    lock.entry(tow_id)
                        .or_insert_with(|| {
                            Arc::new(tokio::sync::Mutex::new(Session {
                                tcp: None,
                                id: tow_id,
                                timeout: tow_timeout,
                                write_cursor: 0,
                                read_cursor: 0,
                                buffer: Vec::new(),
                                closed: false,
                                last_use: Instant::now(),
                            }))
                        })
                        .clone()
                }
            };
            if let Ok(mut session) = session.try_lock_owned() {
                if !session.closed && session.tcp.is_none() {
                    session.tcp = Some(
                        tokio::net::TcpStream::connect(connect_addr)
                            .await
                            .expect("TODO! handle error"),
                    );
                }
                handle_live_session(Direction::WsToTcp, &mut *session, websocket).await;
            } else {
                println!("[{dir} {tow_id:016x}] Erro: sessão já em uso");
            }
        }
        Err(error) => {
            println!("[{:016x}] Erro na conecção do websocket: {error:?}", 0);
        }
    }
}

async fn handle_live_session<S: AsyncRead + AsyncWrite + Unpin>(
    dir: Direction,
    session: &mut Session,
    mut ws: WebSocketStream<S>,
) {
    let id = session.id;
    let kill = match try_handle_live_session(session, &mut ws).await {
        Ok(()) => true,
        Err(SessionError::TcpError(error)) => {
            println!("[{dir} {id:016x}] Conecção tcp encerrada com erro: {error:?}");
            true
        }
        Err(SessionError::WsError(error)) => {
            println!("[{dir} {id:016x}] Conecção ws encerrada com erro: {error:?}");
            false
        }
        Err(SessionError::WsDone) => {
            println!("[{dir} {id:016x}] Conecção ws encerrada");
            false
        }
        Err(SessionError::AckError) => {
            println!("[{dir} {id:016x}] Erro no protocolo (ack invalido)");
            true
        }
    };
    if kill {
        let _ = ws.send(Message::Text(Utf8Bytes::from_static(""))).await;
        let _ = ws.close(None).await;
        session.tcp.take();
        session.timeout = DEFAULT_TIMEOUT_MS;
        session.write_cursor = 0;
        session.read_cursor = 0;
        session.buffer.clear();
        session.closed = true;
        session.last_use = Instant::now();
    }
}

enum SessionError {
    TcpError(std::io::Error),
    WsError(Box<async_tungstenite::tungstenite::Error>),
    WsDone,
    AckError,
}

async fn try_handle_live_session<S: AsyncRead + AsyncWrite + Unpin>(
    session: &mut Session,
    ws: &mut WebSocketStream<S>,
) -> Result<(), SessionError> {
    if session.closed {
        return Ok(());
    }
    let Some(tcp) = session.tcp.as_mut() else {
        return Ok(());
    };

    ws.send(Message::Text(Utf8Bytes::from(
        session.write_cursor.to_string(),
    )))
    .await
    .map_err(Box::new)
    .map_err(SessionError::WsError)?;

    let mut buffer = Vec::new();
    buffer.reserve_exact(1024 * 4);
    buffer.resize(buffer.capacity(), 0);

    let mut session_buffer_read_cursor = None;

    loop {
        if let Some(session_buffer_read_cursor) = &mut session_buffer_read_cursor {
            while *session_buffer_read_cursor < session.buffer.len() {
                let mut slice = &session.buffer[*session_buffer_read_cursor..];
                if slice.len() >= MAX_BYTES_WS_MESSAGE {
                    slice = &slice[..MAX_BYTES_WS_MESSAGE];
                }
                ws.send(Message::Binary(Bytes::copy_from_slice(slice)))
                    .await
                    .map_err(Box::new)
                    .map_err(SessionError::WsError)?;
                *session_buffer_read_cursor += slice.len();
            }
        }
        let select_result = tokio::select! {
            x = tcp.readable() => Left(x),
            x = ws.next() => Right(x),
        };
        match select_result {
            Left(tcp_result) => {
                let bytes_read = tcp_result
                    .and_then(|()| tcp.try_read(&mut buffer))
                    .or_else(|error| {
                        if error.kind() == ErrorKind::WouldBlock {
                            Ok(0)
                        } else {
                            Err(error)
                        }
                    })
                    .map_err(SessionError::TcpError)?;
                session.buffer.extend_from_slice(&buffer[..bytes_read]);
            }
            Right(Some(Ok(ws_message))) => match ws_message {
                Message::Binary(bytes) => {
                    let mut cursor = 0;
                    while cursor < bytes.len() {
                        tcp.writable().await.map_err(SessionError::TcpError)?;
                        cursor += tcp
                            .try_write(&bytes[cursor..])
                            .or_else(|error| {
                                if error.kind() == ErrorKind::WouldBlock {
                                    Ok(0)
                                } else {
                                    Err(error)
                                }
                            })
                            .map_err(SessionError::TcpError)?;
                    }
                }
                Message::Text(utf8_bytes) if utf8_bytes.is_empty() => {
                    return Ok(());
                }
                Message::Text(utf8_bytes) => {
                    if let Ok(ack) = utf8_bytes.parse::<u64>() {
                        if ack < session.read_cursor {
                            return Err(SessionError::AckError);
                        }
                        let delta = ack - session.read_cursor;
                        session.read_cursor += ack;
                        match &mut session_buffer_read_cursor {
                            Some(session_buffer_read_cursor) => {
                                if delta > *session_buffer_read_cursor as u64 {
                                    return Err(SessionError::AckError);
                                }
                                *session_buffer_read_cursor -= delta as usize;
                            }
                            None => {
                                session_buffer_read_cursor = Some(0);
                            }
                        }
                        if delta > session.buffer.len() as u64 {
                            return Err(SessionError::AckError);
                        }
                        session.buffer.copy_within(delta as usize.., 0);
                        session
                            .buffer
                            .truncate(session.buffer.len() - delta as usize);
                        // TODO! remove now unecessary bytes from buffer
                    }
                }
                _ => {}
            },
            Right(Some(Err(ws_error))) => {
                return Err(SessionError::WsError(Box::new(ws_error)));
            }
            Right(None) => {
                return Err(SessionError::WsDone);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn spawn_tcp_over_ws(
    remote_ws_service: *const std::ffi::c_char,
    local_listen: *const std::ffi::c_char,
    timeout: i32,
) -> u16 {
    let remote_ws_service = (!remote_ws_service.is_null())
        .then(|| {
            std::ffi::CStr::from_ptr(remote_ws_service)
                .to_str()
                .unwrap_or("")
        })
        .unwrap_or("");
    let local_listen = (!local_listen.is_null())
        .then(|| {
            std::ffi::CStr::from_ptr(local_listen)
                .to_str()
                .unwrap_or("")
        })
        .unwrap_or("");
    let Ok(connect_request) = remote_ws_service.into_client_request() else {
        return 0;
    };
    let listen = addr::parse_many_socket_addr(local_listen);
    if listen.is_empty() {
        return 0;
    }
    let timeout = if timeout < 0 { 0 } else { timeout as u64 };
    std::thread::spawn(move || {
        let _ = tcp_to_ws_service(connect_request, listen, timeout as u64);
    });
    u16::MAX
}
