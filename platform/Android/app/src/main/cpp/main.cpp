#include <jni.h>

#include "Engine.h"

#include <game-activity/GameActivity.cpp>
#include <game-text-input/gametextinput.cpp>
#include <GLES3/gl3.h>

extern "C" {

#include <game-activity/native_app_glue/android_native_app_glue.c>

/*!
 * Handles commands sent to this Android application
 * @param pApp the app the commands are coming from
 * @param cmd the command to handle
 */
void handle_cmd(android_app *pApp, int32_t cmd) {
    switch (cmd) {
        case APP_CMD_INIT_WINDOW:
            // A new window is created, associate a renderer with it. You may replace this with a
            // "game" class if that suits your needs. Remember to change all instances of userData
            // if you change the class here as a reinterpret_cast is dangerous this in the
            // android_main function and the APP_CMD_TERM_WINDOW handler case.
            if (pApp->userData == nullptr)
                pApp->userData = new Engine(pApp);
            break;
        case APP_CMD_TERM_WINDOW:
            // The window is being destroyed. Use this to clean up your userData to avoid leaking
            // resources.
            // We have to check if userData is assigned just in case this comes in really quickly
            if (pApp->userData != nullptr)
                delete reinterpret_cast<Engine *>(pApp->userData);
            pApp->userData = nullptr;
            break;
        default:
            break;
    }
}
/*!
 * This the main entry point for a native activity
 */
void android_main(struct android_app *pApp) {
    // Can be removed, useful to ensure your code is running

    // register an event handler for Android events
    pApp->onAppCmd = handle_cmd;
    pApp->userData = nullptr;

    // This sets up a typical game/event loop. It will run until the app is destroyed.
    int events;
    android_poll_source *pSource;
    do {
        // Process all pending events before running game logic.
        if (ALooper_pollAll(0, nullptr, &events, (void **) &pSource) >= 0) {
            if (pSource) {
                pSource->process(pApp, pSource);
            }
        }
        if (pApp->userData != nullptr) {
            auto engine = reinterpret_cast<Engine *>(pApp->userData);
            engine->update();
        }
    } while (!pApp->destroyRequested);

}
}
