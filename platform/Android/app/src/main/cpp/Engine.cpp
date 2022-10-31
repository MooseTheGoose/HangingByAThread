//
// Created by John Doe on 10/17/2022.
//

#include <game-activity/native_app_glue/android_native_app_glue.h>
#include <memory>
#include <algorithm>
#include <exception>
#include <GLES3/gl3.h>
#include "Engine.h"

Engine::Engine(struct android_app *appArg) {
    width = height = -1;
    pApp = appArg;
    constexpr EGLint attribs[] = {
            EGL_RENDERABLE_TYPE, EGL_OPENGL_ES3_BIT,
            EGL_SURFACE_TYPE, EGL_WINDOW_BIT,
            EGL_BLUE_SIZE, 8,
            EGL_GREEN_SIZE, 8,
            EGL_RED_SIZE, 8,
            EGL_DEPTH_SIZE, 24,
            EGL_NONE,
    };

    eglDisplay = eglGetDisplay(EGL_DEFAULT_DISPLAY);
    eglInitialize(eglDisplay, nullptr, nullptr);
    int nConfigs;
    eglChooseConfig(eglDisplay, attribs, nullptr, 0, &nConfigs);

    std::unique_ptr<EGLConfig[]> configs(new EGLConfig[nConfigs]);
    eglChooseConfig(eglDisplay, attribs, configs.get(), nConfigs, &nConfigs);

    auto display = eglDisplay;
    auto conf = *std::find_if(
            configs.get(),
            configs.get() + nConfigs,
            [&display](const EGLConfig &curr) {
                EGLint r, g, b, depth;
                return eglGetConfigAttrib(display, curr, EGL_RED_SIZE, &r)
                    && eglGetConfigAttrib(display, curr, EGL_GREEN_SIZE, &g)
                    && eglGetConfigAttrib(display, curr, EGL_BLUE_SIZE, &b)
                    && eglGetConfigAttrib(display, curr, EGL_DEPTH_SIZE, &depth)
                    && r == 8 && g == 8 && b == 8 && depth == 24;
            });
    EGLint fmt;
    eglGetConfigAttrib(eglDisplay, conf, EGL_NATIVE_VISUAL_ID, &fmt);
    eglSurface = eglCreateWindowSurface(eglDisplay, conf, pApp->window, nullptr);
    constexpr EGLint ctxAttrs[] = {EGL_CONTEXT_CLIENT_VERSION, 3, EGL_NONE};
    eglCtx = eglCreateContext(eglDisplay, conf, nullptr, ctxAttrs);
    eglMakeCurrent(eglDisplay, eglSurface, eglSurface, eglCtx);

    glClearColor(145 / 255.0f, 178 / 255.0f, 217 / 255.0f, 1);
}
Engine::~Engine() {
    eglDestroyContext(eglDisplay, eglCtx);
    eglDestroySurface(eglDisplay, eglSurface);
}
extern "C" void bridge_backendSetEngine(Engine *engine);
extern "C" bool bridge_frontendUpdate();
void Engine::update() {
    int newWidth, newHeight;
    bridge_backendSetEngine(this);
    eglQuerySurface(eglDisplay, eglSurface, EGL_WIDTH, &newWidth);
    eglQuerySurface(eglDisplay, eglSurface, EGL_WIDTH, &newHeight);
    if (width != newWidth || height != newHeight) {
        width = newWidth;
        height = newHeight;
        setViewport(0, 0, width, height);
    }
    // If the frontend update fails so catastrophically
    // that this returns false, just terminate.
    if (!bridge_frontendUpdate()) {
        std::terminate();
    }
}
void Engine::clearScreen(float r, float g, float b, float a) {
    glClearColor(r, g, b, a);
    glClear(GL_COLOR_BUFFER_BIT);
}
void Engine::setViewport(int x, int y, int w, int h) {
    glViewport(0, 0, width, height);
}
void Engine::swapBuffers() {
    eglSwapBuffers(eglDisplay, eglSurface);
}
void Engine::getWindowDimensions(int &refWidth, int &refHeight) {
    refWidth = width;
    refHeight = height;
}