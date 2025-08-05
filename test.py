import time
import zmq

context = zmq.Context()
socket = context.socket(zmq.REQ)
socket.connect("tcp://localhost:5555")

while True:
    socket.send_string("generate")
    message = socket.recv().decode("utf-8")
    print(message)
    time.sleep(1)

