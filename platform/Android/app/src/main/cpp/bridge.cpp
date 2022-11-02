//
// Created by John Doe on 10/17/2022.
//

#include "Engine.h"
#include <unistd.h>
#include <exception>
#include <android/asset_manager.h>
#include <android/asset_manager_jni.h>
#include <android/log.h>

#define RETURN_ON_EXCEPT(stmt, val) try { stmt; } catch(...) { return (val); }

extern "C" {
#include <game-activity/native_app_glue/android_native_app_glue.h>

static Engine *m_Engine = nullptr;
static jobject assetManagerRef = nullptr;
static AAssetManager *assetManager = nullptr;

// Most bridge functions can be called from Rust.
// Stack unwinding from when calling C++ from Rust
// is undefined, so we must terminate immediately if
// the bridge throws an exception it cannot handle.
void bridge_backendSetEngine(Engine *newEngine) noexcept {
    m_Engine = newEngine;
}

void bridge_backendInitFromApp(struct android_app *pApp) {
    assetManager = pApp->activity->assetManager;
}

void bridge_backendClearScreen(float r, float g, float b, float a) noexcept {
    RETURN_ON_EXCEPT(m_Engine->clearScreen(r, g, b, a), std::terminate());
}

void bridge_backendSwapBuffers() noexcept {
    RETURN_ON_EXCEPT(m_Engine->swapBuffers(), std::terminate());
}

void bridge_backendWindowDimensions(int *pWidth, int *pHeight) noexcept {
   int width, height;
   RETURN_ON_EXCEPT(m_Engine->getWindowDimensions(width, height), std::terminate());
   *pWidth = width;
   *pHeight = height;
}

AAsset *bridge_backendOpenAsset(const char *fname) noexcept {
    RETURN_ON_EXCEPT(AAssetManager_open(assetManager, fname, AASSET_MODE_STREAMING), nullptr);
}

void bridge_backendCloseAsset(AAsset *aAsset) noexcept {
    RETURN_ON_EXCEPT(AAsset_close(aAsset), std::terminate());
}

int bridge_backendOpenFdAsset(const char *fname, off64_t *pStart, off64_t *pLen) noexcept {
    int fd = -1;
    *pStart = *pLen = 0;
    AAsset *aAsset;
    RETURN_ON_EXCEPT(aAsset = AAssetManager_open(assetManager, fname, AASSET_MODE_UNKNOWN), -1);
    if (aAsset == nullptr)
        return -1;
    try {
        fd = AAsset_openFileDescriptor64(aAsset, pStart, pLen);
    } catch(...) {
        fd = -1;
    };
    try { AAsset_close(aAsset); } catch(...) {}
    if (fd < 0)
        *pStart = *pLen = 0;
    return fd;
}

int bridge_backendReadAsset(AAsset *aAsset, uint8_t *buf, size_t len, size_t *pReadLen) noexcept {
    *pReadLen = 0;
    int sz;
    RETURN_ON_EXCEPT(sz = AAsset_read(aAsset, buf, len), 0);
    if (sz < 0)
        return 0;
    *pReadLen = sz;
    return -1;
}

int bridge_backendSeekAsset(AAsset *aAsset, int from, int64_t to, uint64_t *newPos) noexcept {
    int whence = SEEK_CUR;
    switch (from) {
        case 0: whence = SEEK_SET; break;
        case 1: whence = SEEK_CUR; break;
        case 2: whence = SEEK_END; break;
        default: return 0;
    }
    off64_t ofs;
    RETURN_ON_EXCEPT(ofs = AAsset_seek64(aAsset, to, whence), 0);
    if (ofs == (off64_t)-1)
        return 0;
    *newPos = ofs;
    return -1;
}

};