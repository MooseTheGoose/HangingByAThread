package com.binaryquackers.hbat;

import android.content.Context;
import android.opengl.GLES20;
import android.opengl.GLSurfaceView;
import javax.microedition.khronos.egl.EGLConfig;
import javax.microedition.khronos.opengles.GL10;

public class MainGLSurfaceView extends GLSurfaceView {
    class MainGLRenderer implements GLSurfaceView.Renderer {
        private native void bridgeOnSurfaceCreated();
        private native void bridgeOnDrawFrame();
        private native void bridgeOnSurfaceChanged();

        public void onSurfaceCreated(GL10 unused, EGLConfig cfg) {
            bridgeOnSurfaceCreated();
        }

        public void onDrawFrame(GL10 unused) {
            bridgeOnDrawFrame();
        }

        public void onSurfaceChanged(GL10 unused, int width, int height) {
            bridgeOnSurfaceChanged();
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
