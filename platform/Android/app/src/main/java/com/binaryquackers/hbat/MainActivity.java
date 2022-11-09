package com.binaryquackers.hbat;

import android.opengl.GLSurfaceView;
import android.os.Bundle;
import android.util.Log;
import android.view.Surface;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.app.Activity;

import java.io.File;

public class MainActivity extends Activity {
    private SurfaceView mSurfaceView;
    private native boolean bridgeOnCreate();
    private native boolean bridgeOnDestroy();

    private class MainSurfaceCallback implements SurfaceHolder.Callback {
        private native boolean bridgeSurfaceChanged(Surface s);
        private native boolean bridgeSurfaceDestroyed();
        String TAG = "MainSurfaceCallbacks";
        public void surfaceCreated(SurfaceHolder holder) {
            Log.i(TAG, "surfaceCreated");
        }
        public void surfaceChanged(SurfaceHolder holder, int fmt, int width, int height) {
            Log.i(TAG, "surfaceChanged");
            MainActivity.onPanic(bridgeSurfaceChanged(holder.getSurface()));
        }
        public void surfaceDestroyed(SurfaceHolder holder) {
            Log.i(TAG, "surfaceDestroyed");
            MainActivity.onPanic(bridgeSurfaceDestroyed());
        }
    }

    static {
        System.loadLibrary("hbat");
    }

    static void onPanic(boolean status) {
        if (!status) {
            // Rust code will log into a file
            // or devcon why it crashed. For now,
            // just throw a RuntimeException.
            throw new RuntimeException(
            "Rust code panicked! Crashing immediately!");
        }
    }

    @Override
    protected void onCreate(Bundle savedInstance) {
        super.onCreate(savedInstance);
        onPanic(bridgeOnCreate());
        mSurfaceView = new SurfaceView(this);
        mSurfaceView.getHolder().addCallback(new MainSurfaceCallback());
        setContentView(mSurfaceView);
    }

    @Override
    protected void onDestroy() {
        // Call bridge to destroy things.
        onPanic(bridgeOnDestroy());
        super.onDestroy();
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);

        if (hasFocus) {
            hideSystemUi();
        }
    }

    private void hideSystemUi() {
        View decorView = getWindow().getDecorView();
        decorView.setSystemUiVisibility(
                View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                        | View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                        | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                        | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_FULLSCREEN
        );
    }
}