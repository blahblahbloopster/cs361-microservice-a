# Name Generator
## Building and running
This project is written in rust, so you'll need a copy of cargo.  [rustup](https://rustup.rs) has instructions for all supported platforms, or you can install cargo or rustup through your package manager.  Once you have it, do `cargo run --release` within the cloned folder.  If it doesn't run, ping me.

## Usage
Send the string `"encode"` to the microservice on TCP port 5555, and it will respond with a name.
```python
import time
import zmq

context = zmq.Context()
socket = context.socket(zmq.REQ)
socket.connect("tcp://localhost:5555")

while True:
    socket.send_string("generate")
    message = socket.recv().decode("utf-8")
    print(message)  # prints generated names
    time.sleep(1)
```

## Diagram
![UML sequence diagram](/uml.png)
The microservice includes a separate thread for generating names, which pushes them into a queue continuously.

