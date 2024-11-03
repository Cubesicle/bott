#include <Geode/Geode.hpp>
#include "cubesicle.egui-api/include/api.hpp"
#include "rust-ffi.h"

using namespace geode::prelude;

void record_input(PlayLayer *play_layer, bool pressed, PlayerButton button, bool is_player_1);

$execute {
    egui_api::add_run_fn(gui_run);
}

#include <Geode/modify/GJBaseGameLayer.hpp>
class $modify(GJBaseGameLayer) {
    void handleButton(bool pressed, int button, bool is_player_1) {
        if (!bot_is_recording())
            return GJBaseGameLayer::handleButton(pressed, button, is_player_1);
        
        record_input(PlayLayer::get(), pressed, static_cast<PlayerButton>(button), is_player_1);

        GJBaseGameLayer::handleButton(pressed, button, is_player_1);
    }

    void processCommands(float dt) {
        if (!bot_is_replaying()) return GJBaseGameLayer::processCommands(dt);

        log::debug("{}", m_gameState.m_currentProgress);

        GJBaseGameLayer::processCommands(dt);
    }
};

#include <Geode/modify/PlayLayer.hpp>
class $modify(PlayLayer) {
    void pauseGame(bool p0) {
        PlayLayer::pauseGame(p0);

        if (!bot_is_recording()) return;

        record_input(this, false, PlayerButton::Jump, true);
        record_input(this, false, PlayerButton::Jump, false);
        record_input(this, false, PlayerButton::Left, true);
        record_input(this, false, PlayerButton::Left, false);
        record_input(this, false, PlayerButton::Right, true);
        record_input(this, false, PlayerButton::Right, false);
    }

    void resetLevel() {
        PlayLayer::resetLevel();

        if (!bot_is_recording()) return;

        record_input(this, false, PlayerButton::Jump, true);
        record_input(this, false, PlayerButton::Jump, false);
        record_input(this, false, PlayerButton::Left, true);
        record_input(this, false, PlayerButton::Left, false);
        record_input(this, false, PlayerButton::Right, true);
        record_input(this, false, PlayerButton::Right, false);
    }
};

void record_input(PlayLayer *play_layer, bool pressed, PlayerButton button, bool is_player_1) {
    if (play_layer == nullptr) return;
    if (!play_layer->m_level->isPlatformer() && button != PlayerButton::Jump) return;

    bot_record_input(
        play_layer->m_gameState.m_currentProgress,
        pressed,
        static_cast<int>(button),
        !play_layer->m_level->m_twoPlayerMode || is_player_1
    );
}