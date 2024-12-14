from snakelib import BaseSnakeAi, Direction, GameEnd, SnakeData

class GoUpAi(BaseSnakeAi):
    def __init__(self, name, player_slot='1'):
        super().__init__(name, player_slot)

    def update(self, data: SnakeData):
        print(data.grid)
        print(data.snakes)

        return Direction.UP

if __name__ == "__main__":
    while True:
        try:
            player = GoUpAi("GoUpAi", player_slot='2')
            player.start()

        except GameEnd:
            print("New Game!")
            continue




