#include <jni.h>

#include "Engine.h"

#include <game-activity/GameActivity.cpp>
#include <game-text-input/gametextinput.cpp>
#include <GLES3/gl3.h>

extern "C" {

#include <game-activity/native_app_glue/android_native_app_glue.c>
#include <android/log.h>
#include <pthread.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/stat.h>

bool running = true;

void bridge_frontendDevconEntry();
static void *devconLoop(void *arg) {
    struct android_app *pApp = reinterpret_cast<struct android_app *>(arg);
    int sock = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sock == -1) {
        __android_log_print(ANDROID_LOG_WARN, "HangingByAThread", "Unable to open UNIX socket for devcon\n");
        return NULL;
    }
    sockaddr_un unixAddr = {AF_UNIX};
    snprintf(unixAddr.sun_path, sizeof(unixAddr.sun_path), "%s/devcon", pApp->activity->internalDataPath);
    unlink(unixAddr.sun_path);
    int status = bind(sock, (struct sockaddr *)&unixAddr, sizeof(unixAddr));
    if (status == -1) {
        __android_log_print(ANDROID_LOG_WARN, "HangingByAThread", "Unable to bind to '%s'\n", unixAddr.sun_path);
        close(sock);
        return NULL;
    }
    listen(sock, 1);
    __android_log_print(ANDROID_LOG_INFO, "HangingByAThread", "Waiting for devcon client on UNIX socket '%s'\n", unixAddr.sun_path);
    while (running) {
        int client = accept(sock, nullptr, nullptr);
        if (client == -1) {
            __android_log_print(ANDROID_LOG_WARN, "HangingByAThread",
                                "Failed to accept client for devcon\n");
            continue;
        }
        dup2(client, 2);
        dup2(client, 1);
        dup2(client, 0);
        bridge_frontendDevconEntry();
        close(client);
    }
    close(sock);
    pthread_exit(NULL);
}

bool bridge_frontendInit();
static void initializeRuntime(struct android_app *pApp) {
    static bool initialized = false;
    if (!initialized) {
#ifdef HBAT_DEBUG
        pthread_t thread;
        if(pthread_create(&thread, NULL, devconLoop, pApp)) {
            __android_log_print(ANDROID_LOG_WARN, "HangingByAThread", "Failed to start pthread for devcon\n");
        }
#endif
        if (!bridge_frontendInit()) {
            __android_log_print(ANDROID_LOG_ERROR, "HangingByAThread", "Failed to initialize frontend!\n");
            std::terminate();
        }
    }
}

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
void bridge_backendInitFromApp(struct android_app *pApp);
void android_main(struct android_app *pApp) {
    bridge_backendInitFromApp(pApp);
    initializeRuntime(pApp);

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
