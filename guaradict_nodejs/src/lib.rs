use std::io::{self, Write, Read};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use neon::prelude::*;

#[derive(Clone)]
struct Connection {
    stream: Arc<Mutex<TcpStream>>,
    last_used: Instant,
}

impl Connection {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
            last_used: Instant::now(),
        }
    }
}

enum Event {
    Connected(usize),
    Disconnected(usize),
}

struct GuaradictDriver {
    addr: SocketAddr,
    pool: Arc<Mutex<Vec<Option<Connection>>>>,
    event_sender: mpsc::Sender<Event>,
}

impl GuaradictDriver {
    fn new(addr: SocketAddr, event_sender: mpsc::Sender<Event>) -> Self {
        let driver = Self {
            addr,
            pool: Arc::new(Mutex::new(Vec::with_capacity(10))),
            event_sender,
        };

        let pool_clone = Arc::clone(&driver.pool);
        let event_sender_clone = driver.event_sender.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(10));
                let mut pool = pool_clone.lock().unwrap();
                for (index, conn_option) in pool.iter_mut().enumerate() {
                    if let Some(conn) = conn_option {
                        if conn.last_used.elapsed() > Duration::from_secs(10) {
                            *conn_option = None;
                            let _ = event_sender_clone.send(Event::Disconnected(index));
                        }
                    }
                }
            }
        });

        driver
    }

    fn connect(&self) -> io::Result<usize> {
        let mut pool = self.pool.lock().unwrap();

        if pool.len() < 10 {
            let stream = TcpStream::connect_timeout(&self.addr, Duration::from_secs(30))?;
            let conn = Connection::new(stream);
            pool.push(Some(conn));
            let index = pool.len() - 1;
            self.event_sender.send(Event::Connected(index)).unwrap();
            return Ok(index);
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "Connection pool is full",
        ))
    }

    fn get_connection(&self, index: usize) -> Option<Arc<Mutex<TcpStream>>> {
        let mut pool = self.pool.lock().unwrap();
        if let Some(Some(conn)) = pool.get_mut(index) {
            conn.last_used = Instant::now();
            Some(Arc::clone(&conn.stream))
        } else {
            None
        }
    }

    fn disconnect(&self, index: usize) -> io::Result<()> {
        let mut pool = self.pool.lock().unwrap();
        if index < pool.len() && pool[index].is_some() {
            pool[index] = None;
            self.event_sender.send(Event::Disconnected(index)).unwrap();
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Connection not found",
            ))
        }
    }

    fn set(&self, index: usize, key: String, value: String) -> io::Result<()> {
        if let Some(stream) = self.get_connection(index) {
            let mut stream = stream.lock().unwrap();
            let command = format!("SET {} {}\n", key, value);
            stream.write_all(command.as_bytes())?;
            stream.flush()?;

            let mut buffer = [0; 512];
            stream.read(&mut buffer)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Connection not found",
            ));
        }
        Ok(())
    }

    fn get(&self, index: usize, key: String) -> io::Result<String> {
        if let Some(stream) = self.get_connection(index) {
            let mut stream = stream.lock().unwrap();
            let command = format!("GET {}\n", key);
            stream.write_all(command.as_bytes())?;
            stream.flush()?;

            let mut buffer = [0; 512];
            let n = stream.read(&mut buffer)?;
            let response = String::from_utf8_lossy(&buffer[..n]);
            Ok(response.to_string())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Connection not found",
            ))
        }
    }
}

struct NeonGuaradictDriver {
    inner: Arc<GuaradictDriver>,
}

impl Finalize for NeonGuaradictDriver {}

impl NeonGuaradictDriver {
    fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<NeonGuaradictDriver>> {
        let addr = cx.argument::<JsString>(0)?.value(&mut cx).parse().unwrap();
        let (event_tx, event_rx) = mpsc::channel();
        let driver = GuaradictDriver::new(addr, event_tx);

        let neon_driver = NeonGuaradictDriver {
            inner: Arc::new(driver),
        };

        // Spawn a thread to handle events and send them to JavaScript
        let channel = cx.channel();
        thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                let channel = channel.clone();
                channel.send(move |mut cx| {
                    let js_event = match event {
                        Event::Connected(index) => {
                            let obj = cx.empty_object();
                            let js_type = cx.string("connected");
                            obj.set(&mut cx, "type", js_type).unwrap();
                            let js_index = cx.number(index as f64);
                            obj.set(&mut cx, "index", js_index).unwrap();
                            obj
                        }
                        Event::Disconnected(index) => {
                            let obj = cx.empty_object();
                            let js_type = cx.string("disconnected");
                            obj.set(&mut cx, "type", js_type).unwrap();
                            let js_index = cx.number(index as f64);
                            obj.set(&mut cx, "index", js_index).unwrap();
                            obj
                        }
                    };

                    let global = cx.global_object();
                    let cb: Handle<JsFunction> = global.get(&mut cx, "onEvent")?;
                    let args = vec![js_event.upcast::<JsValue>()];
                    cb.call(&mut cx, global, args)?;

                    Ok(())
                });
            }
        });

        Ok(cx.boxed(neon_driver))
    }

    fn js_connect(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let neon_driver = cx.this_value().downcast_or_throw::<JsBox<NeonGuaradictDriver>, _>(&mut cx)?;
        let (deferred, promise) = cx.promise();

        let driver = Arc::clone(&neon_driver.inner);
        let channel = cx.channel();
        thread::spawn(move || {
            let result = driver.connect();
            deferred.settle_with(&channel, move |mut cx| {
                match result {
                    Ok(index) => Ok(cx.number(index as f64).upcast::<JsValue>()),
                    Err(err) => cx.throw_error(err.to_string()),
                }
            });
        });

        Ok(promise)
    }

    fn js_disconnect(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let neon_driver = cx.this_value().downcast_or_throw::<JsBox<NeonGuaradictDriver>, _>(&mut cx)?;
        let index = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
        let (deferred, promise) = cx.promise();

        let driver = Arc::clone(&neon_driver.inner);
        let channel = cx.channel();
        thread::spawn(move || {
            let result = driver.disconnect(index);
            deferred.settle_with(&channel, move |mut cx| {
                match result {
                    Ok(_) => Ok(cx.undefined().upcast::<JsValue>()),
                    Err(err) => cx.throw_error(err.to_string()),
                }
            });
        });

        Ok(promise)
    }

    fn js_set(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let neon_driver = cx.this_value().downcast_or_throw::<JsBox<NeonGuaradictDriver>, _>(&mut cx)?;
        let index = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
        let key = cx.argument::<JsString>(1)?.value(&mut cx);
        let value = cx.argument::<JsString>(2)?.value(&mut cx);
        let (deferred, promise) = cx.promise();

        let driver = Arc::clone(&neon_driver.inner);
        let channel = cx.channel();
        thread::spawn(move || {
            let result = driver.set(index, key, value);
            deferred.settle_with(&channel, move |mut cx| {
                match result {
                    Ok(_) => Ok(cx.undefined().upcast::<JsValue>()),
                    Err(err) => cx.throw_error(err.to_string()),
                }
            });
        });

        Ok(promise)
    }

    fn js_get(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let neon_driver = cx.this_value().downcast_or_throw::<JsBox<NeonGuaradictDriver>, _>(&mut cx)?;
        let index = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
        let key = cx.argument::<JsString>(1)?.value(&mut cx);
        let (deferred, promise) = cx.promise();

        let driver = Arc::clone(&neon_driver.inner);
        let channel = cx.channel();
        thread::spawn(move || {
            let result = driver.get(index, key);
            deferred.settle_with(&channel, move |mut cx| {
                match result {
                    Ok(val) => Ok(cx.string(val).upcast::<JsValue>()),
                    Err(err) => cx.throw_error(err.to_string()),
                }
            });
        });

        Ok(promise)
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createDriver", NeonGuaradictDriver::js_new)?;
    cx.export_function("connect", NeonGuaradictDriver::js_connect)?;
    cx.export_function("disconnect", NeonGuaradictDriver::js_disconnect)?;
    cx.export_function("set", NeonGuaradictDriver::js_set)?;
    cx.export_function("get", NeonGuaradictDriver::js_get)?;
    Ok(())
}
