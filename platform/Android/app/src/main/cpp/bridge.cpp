//
// Created by John Doe on 10/17/2022.
//

#include "Engine.h"
#include <exception>
#include <android/asset_manager.h>
#include <android/asset_manager_jni.h>
#include <android/log.h>

#define TERM_ON_EXCEPT(stmt) try { stmt; } catch(...) { std::terminate(); }

extern "C" {

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
JNIEXPORT void JNICALL Java_com_binaryquackers_hbat_MainActivity_setAssetManager(JNIEnv *env, jobject activity, jobject amRef) {
    assetManagerRef = env->NewGlobalRef(amRef);
    assetManager = AAssetManager_fromJava(env, assetManagerRef);
}

void bridge_backendClearScreen(float r, float g, float b, float a) noexcept {
    TERM_ON_EXCEPT(m_Engine->clearScreen(r, g, b, a));
}

void bridge_backendSwapBuffers() noexcept {
    TERM_ON_EXCEPT(m_Engine->swapBuffers());
}

void bridge_backendWindowDimensions(int *pWidth, int *pHeight) noexcept {
   int width, height;
   TERM_ON_EXCEPT(m_Engine->getWindowDimensions(width, height));
   *pWidth = width;
   *pHeight = height;
}

AAsset *bridge_backendOpenAsset(const char *fname) {
    return AAssetManager_open(assetManager, fname, AASSET_MODE_STREAMING);
}

void bridge_backendCloseAsset(AAsset *aAsset) {
    AAsset_close(aAsset);
}

int bridge_backendOpenFdAsset(const char *fname, off64_t *pStart, off64_t *pLen) {
    int fd = -1;
    *pStart = *pLen = 0;
    AAsset *aAsset = AAssetManager_open(assetManager, fname, AASSET_MODE_UNKNOWN);
    if (aAsset == nullptr)
        return -1;
    fd = AAsset_openFileDescriptor64(aAsset, pStart, pLen);
    AAsset_close(aAsset);
    if (fd < 0)
        *pStart = *pLen = 0;
    return fd;
}

int bridge_backendReadAsset(AAsset *aAsset, uint8_t *buf, size_t len, size_t *pReadLen) {
    *pReadLen = 0;
    int sz = AAsset_read(aAsset, buf, len);
    if (sz < 0)
        return 0;
    *pReadLen = sz;
    return -1;
}

int bridge_backendSeekAsset(AAsset *aAsset, int from, int64_t to, uint64_t *newPos) {
    int whence = SEEK_CUR;
    switch (from) {
        case 0: whence = SEEK_SET; break;
        case 1: whence = SEEK_CUR; break;
        case 2: whence = SEEK_END; break;
        default: return 0;
    }
    off64_t ofs = AAsset_seek64(aAsset, to, whence);
    if (ofs == (off64_t)-1)
        return 0;
    *newPos = ofs;
    return -1;
}

void bridge_backendLog(const char *msg, size_t len, int lvl) {
    int prio = ANDROID_LOG_DEFAULT;
    switch (lvl) {
        case 0: prio = ANDROID_LOG_INFO; break;
        case 1: prio = ANDROID_LOG_DEBUG; break;
        case 2: prio = ANDROID_LOG_WARN; break;
        case 3: prio = ANDROID_LOG_ERROR; break;
        default: prio = ANDROID_LOG_DEFAULT; break;
    }
    __android_log_print(prio, getprogname(), "%.*s\n", (int)len, msg);
}

};