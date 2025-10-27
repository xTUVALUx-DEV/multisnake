from argparse import ArgumentParser
from threading import Thread
import time
from snakelib import BaseSnakeAi, Direction, GameEnd, SnakeData

class GoUpAi(BaseSnakeAi):
    def __init__(self, name, player_slot='1'):
        self.last_dir = Direction.LEFT
        super().__init__(name, player_slot)

    def update(self, data: SnakeData):
        print(data.grid)
        print(data.snakes)
        me = data.me.id
        print(data.me)
    
        for y in range(data.grid.height):
            for x in range(data.grid.width):
                if data.grid.data[x][y] == me:
                    print(x,y)
                    if x < 2:
                        self.last_dir = Direction.RIGHT
                        return self.last_dir
                    elif x > data.grid.width - 3:
                        self.last_dir = Direction.LEFT
                        return self.last_dir

        return self.last_dir
    

def run(slot):
    while True:
        try:
            player = GoUpAi("DontCrashAi", player_slot=slot)
            player.start()

        except GameEnd:
            print("New Game!")
            continue

if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument('Player Slot', default='1', nargs='?')
    parser.add_argument('-n', type=int, default=None, nargs='?')
    args = vars(parser.parse_args())
    
    if args['n'] != None:
        threads = [Thread(target=run, args=(i+1,), daemon=True) for i in range(args['n'])]
        [t.start() for t in threads]
        
        while True:
            time.sleep(0.5)

        sys.exit()

    run(args['Player Slot'])
