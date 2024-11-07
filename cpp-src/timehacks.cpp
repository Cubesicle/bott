#include <Geode/Geode.hpp>
#include "rust-ffi.h"

using namespace geode::prelude;

const int tps = 240;

#include <Geode/modify/GJBaseGameLayer.hpp>
class $modify(GJBaseGameLayer) {
    void update(float dt) {
        if (bot_frame_stepper_is_on()) {
            dt = 1.0 / tps;
            if (!bot_frame_stepper_should_advance()) return;
            bot_set_frame_stepper_advance(false);
        }

        GJBaseGameLayer::update(dt);
    }
};