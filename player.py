import os
import time
import win32pipe
import win32file

PIPE_NAME = r'\\.\pipe\SnakePipe'

class SnakePlayer:
    def __init__(self):
        while True:
            try:
                pipe = win32file.CreateFile(
                    PIPE_NAME,
                    win32file.GENERIC_READ | win32file.GENERIC_WRITE,
                    0,
                    None,
                    win32file.OPEN_EXISTING,
                    0,
                    None
                )
                break
            except Exception as e:
                print("Waiting for pipe to be ready...")
                time.sleep(1)
                

        print("Connected to named pipe.")
        try:
            # Send a message to the Rust program
            while True:
                message = "Hello from Python"
                win32file.WriteFile(pipe, message.encode())
                time.sleep(1)

            # Receive a response
            response = win32file.ReadFile(pipe, 64 * 1024)
            print("Received:", response[1].decode())
        finally:
            win32file.CloseHandle(pipe)


player = SnakePlayer()




