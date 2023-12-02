TLS is now a thing !!!!
Stream is dynamically created based on config TLS or Clear  
```rust
// Trait to define a custom stream with Read, Write, Sync, Send, and Debug capabilities
trait CustomStreamTrait: Read + Write + Sync + Send + Debug {}

// Implement CustomStreamTrait for TcpStream
impl CustomStreamTrait for TcpStream {}

// Implement CustomStreamTrait for SslStream<TcpStream>
impl CustomStreamTrait for SslStream<TcpStream> {}

#[derive(Debug)]
// Define a struct `Streams` that wraps a boxed dynamic trait object `CustomStreamTrait`
struct Streams(Box<dyn CustomStreamTrait>);

// Struct to represent a connection to the Twitch server
#[derive(Debug, Clone)]
pub struct TwitchConnection {
    stream: Arc<Mutex<Streams>>,
    pub callbacks: Arc<Mutex<TwitchCallbacks>>,
}
```

Then it is simply created as :

```rust
    // Set the SSL stream to non-blocking mode
    ssl_stream.get_ref().set_nonblocking(true).unwrap();
    // Wrap the SSL stream in a Streams struct and box it
    Streams(Box::new(ssl_stream))
//
//   OR 
//
    // Set the TCP stream to non-blocking mode
    tcp_stream.set_nonblocking(true).unwrap();
    // Wrap the TCP stream in a Streams struct and box it
    Streams(Box::new(tcp_stream))
//
//   Then Arc Mutex are generated for sharing SSL/TLS stream that does not implement Clone
//   To manage "Clear" TcpStreeam it is also converted to Arc Mutex, 
//   TcpStreeam implement try_clone() that it is not usable with SslStream as for reason before.
//   For this reason both are cloned as Arc 
//
    // Clone the stream for use in the internal thread
    let stream_int = Arc::new(Mutex::new(streams));
    // Clone the stream for use as return value
    let stream_ret = stream_int.clone();

```


Both Streams can be passed to the same management logic without duplicating a code for different stream.