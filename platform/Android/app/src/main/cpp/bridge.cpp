//
// Created by John Doe on 10/17/2022.
//

#include "Engine.h"

extern "C" {

static Engine *m_Engine = nullptr;

void bridge_backendSetEngine(Engine *newEngine) {
    m_Engine = newEngine;
}
void bridge_backendClearScreen(float r, float g, float b, float a) {
    m_Engine->clearScreen(r, g, b, a);
}
void bridge_backendSwapBuffers() {
    m_Engine->swapBuffers();
}
void bridge_backendWindowDimensions(int *pWidth, int *pHeight) {
   int width, height;
   m_Engine->getWindowDimensions(width, height);
   *pWidth = width;
   *pHeight = height;
}

};