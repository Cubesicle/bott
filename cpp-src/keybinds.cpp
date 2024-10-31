#include <Geode/Geode.hpp>
#include "rust-ffi.h"
using namespace geode::prelude;

#ifndef GEODE_IS_ANDROID
    #include <geode.custom-keybinds/include/Keybinds.hpp>
    $execute {
        keybinds::BindManager::get()->registerBindable({
            "toggle_gui"_spr,
            "Toggle GUI",
            "Hides/unhides the GUI",
            { keybinds::Keybind::create(KEY_OEMPeriod, keybinds::Modifier::None) },
            "Bott"
        });
        new EventListener([=](keybinds::InvokeBindEvent* event) {
            if (event->isDown()) show_gui(!gui_is_showing());
	        return ListenerResult::Propagate;
        }, keybinds::InvokeBindFilter(nullptr, "toggle_gui"_spr));
    }
#endif