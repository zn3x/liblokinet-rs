use std::{
    ffi::{CStr, CString},
    os::raw::c_char
};

use tokio::{
    task,
    sync::{mpsc::{self, UnboundedSender, UnboundedReceiver}, oneshot}, net::TcpStream
};

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct lokinet_context {
    _inner: [u8;0]
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct lokinet_stream_result {
    error: i32,
    local_address: [c_char; 256],
    local_port: i32,
    stream_id: i32
}

#[link(name="lokinet", kind="dylib")]
extern {
    fn lokinet_context_new() -> *const lokinet_context;
    fn lokinet_context_free(ctx: *const lokinet_context);
    fn lokinet_context_start(ctx: *const lokinet_context) -> i32;
    fn lokinet_status(ctx: *const lokinet_context) -> i32;
    //fn lokinet_wait_for_ready(n: u32, ctx: *const lokinet_context) -> i32;
    fn lokinet_context_stop(ctx: *const lokinet_context);
    fn lokinet_add_bootstrap_rc(seed: *const u8, size: usize, ctx: *const lokinet_context);

    fn lokinet_outbound_stream(
        res: *const lokinet_stream_result,
        remote_addr: *const c_char,
        local_addr: *const c_char,
        ctx: *const lokinet_context);
}


enum CtxEvent {
    Start(oneshot::Sender<i32>),
    Stop,
    Status(oneshot::Sender<i32>),
    Exit,
    BootstrapRc(Vec<u8>),
    NewStream(String, oneshot::Sender<Result<String, i32>>)
}

pub struct Context {
    tx: UnboundedSender<CtxEvent>
}

impl Context {
    pub fn new() -> Context {
        let (tx, rx) = mpsc::unbounded_channel();
        let context  = Context { tx };

        task::spawn_blocking(move || {
            Context::event_handler(rx);
        });
        
        context
    }

    pub async fn bootstrap_rc(&mut self, seed: &[u8]) {
        self.tx.send(CtxEvent::BootstrapRc(seed.to_vec())).ok();
    }

    pub async fn start(&mut self) -> i32 {
        let (tx, rx) = oneshot::channel();
        self.tx.send(CtxEvent::Start(tx)).ok();

        return rx.await.unwrap();
    }

    pub async fn stop(&mut self) {
        self.tx.send(CtxEvent::Stop).ok();
    }

    pub async fn status(&mut self) -> i32 {
        let (tx, rx) = oneshot::channel();
        self.tx.send(CtxEvent::Status(tx)).ok();

        return rx.await.unwrap();
    }

    pub async fn new_tcp_stream(&mut self, dest: &str) -> Result<TcpStream, std::io::Error> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(CtxEvent::NewStream(dest.to_owned(), tx)).ok();

        let addr = rx.await.unwrap();
        println!("woowowowow {:?}", addr);
        
        TcpStream::connect(addr.unwrap()).await
    }

    fn event_handler(mut rx: UnboundedReceiver<CtxEvent>) {
        let ctx;
        unsafe {
            ctx = lokinet_context_new();
        }
        unsafe {
            loop {
                match rx.blocking_recv() {
                    Some(CtxEvent::Start(tx)) => {
                        tx.send(lokinet_context_start(ctx)).ok();
                    },
                    Some(CtxEvent::Stop) => {
                        lokinet_context_stop(ctx);
                    },
                    Some(CtxEvent::Status(tx)) => {
                        tx.send(lokinet_status(ctx)).ok();
                    },
                    Some(CtxEvent::Exit) => {
                        lokinet_context_free(ctx);
                        return;
                    },
                    Some(CtxEvent::BootstrapRc(seed)) => {
                        lokinet_add_bootstrap_rc(seed.as_ptr(), seed.len(), ctx);
                    },
                    Some(CtxEvent::NewStream(dest, tx)) => {
                        let mut stream = lokinet_stream_result {
                            error: 0,
                            local_address: [0; 256],
                            local_port: 0,
                            stream_id: 0
                        };

                        let addr = CString::new(dest).unwrap();
                        lokinet_outbound_stream(&mut stream, addr.as_ptr(), std::ptr::null(), ctx);
                        
                        match stream.error {
                            0       => {
                                let host = CStr::from_ptr(stream.local_address.as_ptr());
                                tx.send(Ok(format!("{}:{}", host.to_str().unwrap(), stream.local_port))).ok();
                            },
                            err @ _ => {
                                tx.send(Err(err)).ok();
                            }
                        }
                    },
                    None => ()
                }
            }
        }
    }

    pub async fn connect() {
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        self.tx.send(CtxEvent::Exit).ok();
    }
}
