package com.binaryquackers.hbat;

import android.content.Context;
import android.opengl.GLES20;
import android.opengl.GLSurfaceView;
import javax.microedition.khronos.egl.EGLConfig;
import javax.microedition.khronos.opengles.GL10;

public class MainGLSurfaceView extends GLSurfaceView {
    class MainGLRenderer implements GLSurfaceView.Renderer {
        private native boolean bridgeOnSurfaceCreated();
        private native boolean bridgeOnDrawFrame();
        private native boolean bridgeOnSurfaceChanged();

        public void onSurfaceCreated(GL10 unused, EGLConfig cfg) {
            MainActivity.onPanic(bridgeOnSurfaceCreated());
        }

        public void onDrawFrame(GL10 unused) {
            MainActivity.onPanic(bridgeOnDrawFrame());
        }

        public void onSurfaceChanged(GL10 unused, int width, int height) {
            MainActivity.onPanic(bridgeOnSurfaceChanged());
        }
    }

    MainGLRenderer renderer;
    public MainGLSurfaceView(Context ctx) {
        super(ctx);

        setEGLContextClientVersion(2);

        renderer = new MainGLRenderer();
        setRenderer(renderer);
        setRenderMode(RENDERMODE_CONTINUOUSLY);
    }
}
