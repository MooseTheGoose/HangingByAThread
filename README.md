# Hanging by a Thread
A 3D MMBN-like mobile game

# How to build the game
The frontend libs for the game are written in
Rust and the backend libs are written in C++
and any platform-specific language for mobile.  

Counterintuitivey, the frontend libraries must be built first.
To do this you run the 'package.py' script with Python.
After that, the Rust libraries will be built and ready
to link in the OS-specific package manager.  

After that, go to the 'platform' folder and you'll
see the android folder (ios folder will be later implemented).

These folders contain OS-specific projects that you can
open up in their IDEs (Android Studio, XCode) and just
build and run them like you would any other project.
