#
# Copyright (C) The Android Open Source Project
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

cmake_minimum_required(VERSION 3.4.1)

find_package(game-activity REQUIRED CONFIG)

# build native_app_glue as a static lib
set(${CMAKE_C_FLAGS}, "${CMAKE_C_FLAGS}")

# now build app's shared lib
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=gnu++11 -Wall -Werror")

#target_include_directories(native-activity PRIVATE
#    ${ANDROID_NDK}/sources/android/native_app_glue)

get_target_property(game-activity-include
                    game-activity::game-activity
                    INTERFACE_INCLUDE_DIRECTORIES)

add_library(native-activity SHARED
            main.cpp
            ${game-activity-include}/game-activity/native_app_glue/android_native_app_glue.c
	    ${game-activity-include}/game-activity/GameActivity.cpp
	    ${game-activity-include}/game-text-input/gametextinput.cpp)

# add lib dependencies
target_link_libraries(native-activity
    game-activity::game-activity
    android
    EGL
    GLESv1_CM
    log)
