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

if __name__ == "__main__":
    while True:
        try:
            player = GoUpAi("DontCrashAi", player_slot='1')
            player.start()

        except GameEnd:
            print("New Game!")
            continue



