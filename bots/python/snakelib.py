import enum
import sys
import os
import time
from turtle import right
import struct
import socket

if os.name == 'nt':
    import win32file

class OsMode(enum.Enum):
    WINDOWS = 0
    LINUX = 1

OSMODE = OsMode.LINUX

PIPE_BASE_NAME = r'\\.\pipe\SnakePipe'
SOCK_BASE_NAME = '/tmp/multisnake'

class GameEnd(Exception):
    pass

class Direction(enum.Enum):
    UP = 10
    DOWN = 11
    LEFT = 12
    RIGHT = 13

class SnakeState(enum.Enum):
    ALIVE = 1
    DEAD = 0

class BufferReader:
    def __init__(self, buffer):
        self.buff = buffer

    def read_int(self):
        out = struct.unpack("i", self.buff[:4])[0]
        self.buff = self.buff[4:]
        return out

    def read_uint(self):
        out = struct.unpack("I", self.buff[:4])[0]
        self.buff = self.buff[4:]
        return out

    def read_short(self):
        out = struct.unpack("h", self.buff[:2])[0]
        self.buff = self.buff[2:]
        return out

    def read_ushort(self):
        out = struct.unpack("H", self.buff[:2])[0]
        self.buff = self.buff[2:]
        return out
    
    def read_ubyte(self):
        out = struct.unpack("B", self.buff[:1])[0]
        self.buff = self.buff[1:]
        return out

    def read_string(self):
        length = self.read_ushort()
        out = self.buff[:length].decode()
        self.buff = self.buff[length:]
        return self

class GameGrid:
    def __init__(self, grid_data, height, width):
        """
        Ein Grid ist eine 2-Dimenstionale Array mit folgenden Werten:
        -1 = Food
        0  = Nichts
        10 = Snake 1 (id=10)
        11 = Snake 2 (id=11)
        ...
        """
        self.height = height
        self.width = width
        self.data = grid_data
    
    def __repr__(self):
        outstr = ""
        for y in range(self.height):
            for x in range(self.width):
                outstr += "  "+str(self.data[x][y])
            outstr += "\n"
        return outstr

    def get_data(self):
        return self.data

class Snake:
    def __init__(self, id, name, state, tiles, head, max_size):
        self.id = id
        self.name = name
        self.max_size = max_size
        self.state = SnakeState(state)
        self.tiles = tiles
        self.head = head
    
    def __repr__(self):
        return f"<Snake name={self.name} id={self.id} state={self.state}>"

class SnakeData:
    def __init__(self, buffer):
        self.height = struct.unpack('H', buffer[:2])[0]  # Todo: Use the BufferReader
        self.width = struct.unpack('H', buffer[2:4])[0]
        my_snake_id = struct.unpack('H', buffer[4:6])[0] + 10

        self.raw_grid: list[list[int]] = []
        self.snakes: dict[int, Snake] = {}
        self.me: Snake = None

        for x in range(self.width):
            inner = []
            for y in range(self.height):
                index = 6+(x+y*self.width)*2
                inner.append(struct.unpack('h', buffer[index:index+2])[0])
            self.raw_grid.append(inner)
        
        start_snakes = 6+(self.height*self.width)*2
        num_snakes = struct.unpack('H', buffer[start_snakes:start_snakes+2])[0]
        reader = BufferReader(buffer[start_snakes + 2:])
        for i in range(num_snakes):
            snake_id = reader.read_short() + 10
            snake_name = reader.read_string()
            max_size = reader.read_ushort()
            tiles_len = reader.read_ushort()
            tiles = []
            for i in range(tiles_len):
                tiles.append(reader.read_ushort())

            alive = reader.read_ubyte()
            if len(tiles) == 0:
                continue
            
            head = (tiles[0] % self.width, tiles[0] // self.width)

            snake = Snake(snake_id, snake_name, alive, tiles, head, max_size)
            self.snakes[snake_id] = snake
            if snake_id == my_snake_id:
                self.me = snake

        self.grid: GameGrid = GameGrid(self.raw_grid, self.height, self.width)

class BaseSnakeAi:
    def __init__(self, name, player_slot='1'):
        """
        To use multiple ais give each of them different 'player_slot's.
        1-12 are possible.
        """
        self.name = name
        self.player_slot = str(player_slot)
        self.current_markes_cells_packet = None
        
    def start(self):
        print("Waiting for game...")
        print(PIPE_BASE_NAME + self.player_slot)
        while True:
            time.sleep(1)
            try:
                if OSMODE == OsMode.WINDOWS:
                    self.pipe = win32file.CreateFile(
                        PIPE_BASE_NAME + self.player_slot,
                        win32file.GENERIC_READ | win32file.GENERIC_WRITE,
                        0,
                        None,
                        win32file.OPEN_EXISTING,
                        0,
                        None
                    )
                    break
                else:
                    self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                    self.sock.connect(SOCK_BASE_NAME + self.player_slot + ".sock")
                    break
            except KeyboardInterrupt:
                sys.exit()
            except Exception as e:
                time.sleep(1)
                continue

        print(f"Connected as {self.name}")

        if OSMODE == OsMode.WINDOWS:
            win32file.WriteFile(self.pipe, self.name.encode())
        else:
            self.sock.sendall(self.name.encode())

        while True:
            try:
                if OSMODE == OsMode.WINDOWS:
                    response = win32file.ReadFile(self.pipe, 64 * 1024)
                else:
                    response = (0, self.sock.recv(1024))
                    print(response)
            except:
                raise GameEnd
            
            if response[0] == 0:
                buffer  = response[1]
                if len(buffer) <= 0:
                    break

                if buffer[0] == 0:
                    try:
                        data = SnakeData(buffer[1:])
                        self.me = data.me
                        direction: Direction = self.update(data)
                    except Exception as e:
                        print("[ERROT]", e)
                        raise e
                        # continue
                    
                    try:
                        packet = struct.pack("B", direction.value)
                        if self.current_markes_cells_packet is not None:
                            packet += self.current_markes_cells_packet
                            self.current_markes_cells_packet = None

                        if OSMODE == OsMode.WINDOWS:
                            win32file.WriteFile(self.pipe, packet)
                        else:
                            self.sock.sendall(packet)
                    except Exception as e:
                        print(e)
                        raise GameEnd
                    
                elif buffer[0] == 2:
                    winner_id = struct.unpack("i", buffer[1:5])[0] + 10
                    self.on_gameend(winner_id)

    def send_marked_cells(self, cells):
        packet = b'\x14' + struct.pack('H', len(cells)) + b''.join([struct.pack('H', x) for x in cells])
        self.current_markes_cells_packet = packet

    def on_gameend(self, winner_id):
        print(f"Player with id {winner_id} won")

    def update(self, data: SnakeData) -> Direction:
        raise NotImplementedError("Du musst die update methode überschreiben")

if __name__ == "__main__":
    while True:
        try:
            player = BaseSnakeAi("PyAi", player_slot='1')
            player.start()
        except GameEnd:
            print("New Game!")
            continue

