import time
from queue import Queue 
from threading import Thread 
import os
import socket
import json
import matplotlib.pyplot as plt
import numpy as np


def ipc_receiver(queue_sender):
    # Path for the Unix socket.
    SOCKET_PATH = "/tmp/fmcw_ipc_socket"

    # Ensure the socket does not already exist.
    if os.path.exists(SOCKET_PATH):
        os.remove(SOCKET_PATH)

    # Create a Unix socket.
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server_socket:
        server_socket.bind(SOCKET_PATH)
        server_socket.listen(1)
        print("Python server listening...")

        conn, _ = server_socket.accept()
        with conn:
            print("Connection established.")
            buffer = ""

            while True:
                # Read data from the client.

                data = conn.recv(1024).decode("utf-8")
                if not data:
                    break

                # Accumulate data and split by newline for complete messages.
                buffer += data
                while "\n" in buffer:
                    message, buffer = buffer.split("\n", 1)
                    
                    # Deserialize the message.
                    try:
                        message = json.loads(message)
                        print(f"Received from Rust: {message}")

                        # Process the message (user-defined logic goes here).
                        queue_sender.put(message)

                    except json.JSONDecodeError:
                        print("Invalid message format received.")

def data_renderer(queue_receiver):
    while True:
        data = queue.get()

        frame_num = data["frame_num"]
        if "pointcloud" in data:
            pc = data["pointcloud"]
            pass
        if "range_profile" in data:
            rp = data["range_profile"]

def draw_scatter(pointcloud_queue: Queue):
    # Plot interactive on
    plt.ion()
    fig, ax = plt.subplots()
    x, y = [],[]
    sc = ax.scatter(x,y)
    plt.xlim(-10,10)
    plt.ylim(0,20)

    plt.draw()
    while True:
        if pointcloud_queue.empty():
            time.sleep(0.02)
            continue
        pointcloud = pointcloud_queue.get()
        x= [point['x'] for point in pointcloud]
        y= [point['y'] for point in pointcloud]
        sc.set_offsets(np.c_[x,y])
        fig.canvas.draw_idle()
        plt.pause(0.1)

    plt.waitforbuttonpress()

if __name__ == "__main__":
    queue = Queue()
    ipc_thread = Thread(target=ipc_receiver, args=(queue,))
    ipc_thread.start()
    renderer_thread = Thread(target=data_renderer, args=(queue,))
    renderer_thread.start()
    ipc_thread.join()
    renderer_thread.join()

