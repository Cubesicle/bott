using namespace geode::prelude;

std::string save_dir;

extern "C" {
    #include "ffi.h"

    const char *get_save_dir() {
        if (save_dir.empty()) save_dir = Mod::get()->getSaveDir().string();
        return save_dir.c_str();
    }
    
    void log_debug(const char *str) { log::debug("{}", str); }
    void log_info(const char *str) { log::info("{}", str); }
    void log_warn(const char *str) { log::warn("{}", str); }
    void log_error(const char *str) { log::error("{}", str); }
}