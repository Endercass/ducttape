# DuctTape

An open-source game made for hack club [Juice](https://github.com/hackclub/juice). 
This game is a dungeon/cave crawler puzzle platformer, 
where you craft items to solve "puzzles" that are integrated into the caves and needed to navigate the environment.

I am likely not going to be able to finish this game by the deadline, but I want to complete it and build it out to be a more advanced game in the future.

## ducttape-item-engine

The internal engine used to manage items, striving to work idependently from the overarching game engine.

## ducttape-native

All of the game code is stored here, and is loaded as a gdextension in the godot project

## ducttape-godot

The godot game project

## Building

1. Build the rust project:
```sh
# in the checkout folder
cargo build
```

2. Open the godot project and hit run
