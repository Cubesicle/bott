#include <Geode/Geode.hpp>
#include "cubesicle.egui-api/include/api.hpp"
#include "rust-ffi.h"

using namespace geode::prelude;

$execute {
    egui_api::add_run_fn(gui_run);
}

#include <Geode/modify/PlayLayer.hpp>
class $modify(PlayLayer) {
    void resetLevel() {
        log::debug("lol");
        PlayLayer::resetLevel();
    }
};