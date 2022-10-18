#ifndef HANGINGBYATHREAD_ENGINE_H
#define HANGINGBYATHREAD_ENGINE_H

#include <EGL/egl.h>

typedef struct android_app android_app;

class Engine {
private:
    android_app *pApp;
    EGLDisplay eglDisplay;
    EGLSurface eglSurface;
    EGLContext eglCtx;
    int width, height;
public:
    Engine(android_app *pApp);
    ~Engine();

    void update();
    void getWindowDimensions(int& width, int& height);
    void setViewport(int x, int y, int w, int h);
    void clearScreen(float r, float g, float b, float a);
    void swapBuffers();
};


#endif //HANGINGBYATHREAD_ENGINE_H
