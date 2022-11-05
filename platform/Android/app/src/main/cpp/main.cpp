#include <jni.h>


extern "C" {
// I have to use something from the static library.
// Otherwise, it will get optimized out.
extern void hbat_bridge_stub();
void hbat_unused_function() { hbat_bridge_stub(); }
}