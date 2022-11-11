# Hanging by a Thread
A 3D MMBN-like mobile game for iOS (one day...) and Android. 
Note that this project is a learning exercise for
both graphics and Rust development on mobile, so I
forgo a lot of libraries and tools I'd normally use
and "reimplement the wheel". A lot...

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

Note that, at the moment, these projects won't
call package.py for you. You still have to do
that separately. They should, but I haven't
bothered with that yet...

# How to use the developer console
This doesn't exist right now. It used to and likely will again,
but I've just been too busy doing annoying platform stuff at the moment.

On Android, the console will create a UNIX socket
and wait for connections. To connect, you must use
the following adb command.
  
```
adb shell run-as com.binaryquackers.hbat nc -U files/devcon
```
  
You should then see a '$' prompting you for input.
  
You can only have one session at a time, but you may
restart the session as many times as you want.
  
At the moment, all this does is echo back what you input.
The console can still be useful because it redirects stdin,
stdout, and stderr, so on a panic, it will dump the error
to your current console instead of null, which isn't useful,
and log, which would be too much clutter. 
