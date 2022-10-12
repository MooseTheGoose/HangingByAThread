/*
 * Copyright (C) 2010 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

//BEGIN_INCLUDE(all)
#include <EGL/egl.h>
#include <GLES/gl.h>
#include <android/log.h>
#include <game-activity/native_app_glue/android_native_app_glue.h>

class HBATEngine {
};

extern "C" {
    void android_main(struct android_app* state);
};
static void android_app_cmd(struct android_app *app, int32_t cmd) {
}
void android_main(struct android_app* app) {
    android_app_set_key_event_filter(app, NULL);
    android_app_set_motion_event_filter(app, NULL);
    app->userData = new HBATEngine;
    app->onAppCmd = android_app_cmd;
    bool running = true;
    while (running) {
	    /*
        int events;
	struct android_poll_source* source;
	while (ALooper_pollAll(0, NULL, &events, (void**)&source) >= 0) {
            if (source != NULL) {
                source->process(source->app, source);
            }
	    if (app->destroyRequested) {
                running = false;
		break;
	    }
	}
        android_input_buffer *inputBuffer = android_app_swap_input_buffers(app);
        android_app_clear_motion_events(inputBuffer);
        android_app_clear_key_events(inputBuffer);
	*/
    }
    // Cleanup any OS resources...
}
