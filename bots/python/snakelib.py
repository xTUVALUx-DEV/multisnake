import enum
import os
import time
from turtle import right
import win32file
import struct

PIPE_BASE_NAME = r'\\.\pipe\SnakePipe'

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
    def __init__(self, id, name, state):
        self.id = id
        self.name = name
        self.state = SnakeState(state)
    
    def __repr__(self):
        return f"<Snake name={self.name} id={self.id} state={self.state}>"

class SnakeData:
    def __init__(self, buffer, own_name):
        self.height = struct.unpack('H', buffer[:2])[0]
        self.width = struct.unpack('H', buffer[2:4])[0]
        self.raw_grid: list[list[int]] = []
        self.snakes: dict[int, Snake] = {}
        self.me: Snake = None

        for x in range(self.width):
            inner = []
            for y in range(self.height):
                index = 4+(x+y*self.width)*2
                inner.append(struct.unpack('h', buffer[index:index+2])[0])
            self.raw_grid.append(inner)
        
        start_snakes = 4+(self.height*self.width)*2
        num_snakes = struct.unpack('H', buffer[start_snakes:start_snakes+2])[0]
        
        curr = start_snakes + 2
        for i in range(num_snakes):
            snake_id = struct.unpack('h', buffer[curr:curr+2])[0] + 10
            curr += 2
            len_name = struct.unpack('H', buffer[curr:curr+2])[0]
            curr += 2
            name = buffer[curr:curr+len_name].decode()
            curr += len_name
            alive = buffer[curr]
            curr += 1

            snake = Snake(snake_id, name, alive)
            self.snakes[snake_id] = snake
            if name == own_name:
                self.me = snake

        self.grid: GameGrid = GameGrid(self.raw_grid, self.height, self.width)

class BaseSnakeAi:
    def __init__(self, name, player_slot='1'):
        """
        To use multiple ais give each of them different 'player_slot's.
        1-4 are possible.
        """
        self.name = name
        self.player_slot = player_slot
        
    def start(self):
        while True:
            try:
                print(PIPE_BASE_NAME + self.player_slot)
                pipe = win32file.CreateFile(
                    PIPE_BASE_NAME + self.player_slot,
                    win32file.GENERIC_READ | win32file.GENERIC_WRITE,
                    0,
                    None,
                    win32file.OPEN_EXISTING,
                    0,
                    None
                )
                break
            except Exception as e:
                print("Waiting for game...")
                time.sleep(1)

        print(f"Connected as {self.name}")

        win32file.WriteFile(pipe, self.name.encode())
        while True:
            try:
                response = win32file.ReadFile(pipe, 64 * 1024)
            except:
                raise GameEnd
            
            if response[0] == 0:
                direction: Direction = self.update(SnakeData(response[1], self.name))
                win32file.WriteFile(pipe, struct.pack("B", direction.value))

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

