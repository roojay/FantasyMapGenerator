#include "render.h"
#include <fstream>

void gen::render::drawMap(std::string &drawdata, std::string filename) {
    // Output JSON data for WebGPU rendering
    std::ofstream file(filename);
    file << std::string(drawdata.data());
    file.close();
}
