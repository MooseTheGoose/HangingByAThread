package com.binaryquackers.hbat;

import android.content.Context;
import android.opengl.GLES20;
import android.opengl.GLSurfaceView;
import javax.microedition.khronos.egl.EGLConfig;
import javax.microedition.khronos.opengles.GL10;

public class MainGLSurfaceView extends GLSurfaceView {
    class MainGLRenderer implements GLSurfaceView.Renderer {
        private native void bridgeOnSurfaceCreated(Context ctx);
        private Context ctx;

        public MainGLRenderer(Context ctx) {
            this.ctx = ctx;
        }

        public void onSurfaceCreated(GL10 unused, EGLConfig cfg) {
            bridgeOnSurfaceCreated(ctx);
            GLES20.glClearColor(0.0f, 0.0f, 0.0f, 1.0f);
        }

        public void onDrawFrame(GL10 unused) {
            GLES20.glClear(GLES20.GL_COLOR_BUFFER_BIT);
        }

        public void onSurfaceChanged(GL10 unused, int width, int height) {
            GLES20.glViewport(0, 0, width, height);
        }
    }

    MainGLRenderer renderer;
    public MainGLSurfaceView(Context ctx) {
        super(ctx);

        setEGLContextClientVersion(2);

        renderer = new MainGLRenderer(ctx);
        setRenderer(renderer);
        setRenderMode(RENDERMODE_CONTINUOUSLY);
    }
}
