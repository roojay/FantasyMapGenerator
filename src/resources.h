#ifndef RESOURCES_H
#define RESOURCES_H

#define RESOURCES_EXECUTABLE_DIRECTORY 	"/home/runner/work/FantasyMapGenerator/FantasyMapGenerator/build"
#define RESOURCES_FONT_DATA_DIRECTORY 	"/home/runner/work/FantasyMapGenerator/FantasyMapGenerator/build/fontdata"
#define RESOURCES_CITY_DATA_DIRECTORY 	"/home/runner/work/FantasyMapGenerator/FantasyMapGenerator/build/citydata"
#define RESOURCES_FONT_DATA_RESOURCE 	"/home/runner/work/FantasyMapGenerator/FantasyMapGenerator/build/fontdata/fontdata.json"
#define RESOURCES_CITY_DATA_RESOURCE 	"/home/runner/work/FantasyMapGenerator/FantasyMapGenerator/build/citydata/countrycities.json"

#include <string>

namespace gen{
namespace resources {
    
extern std::string getExecutableDirectory();
extern std::string getFontDataDirectory();
extern std::string getCityDataDirectory();
extern std::string getFontDataResource();
extern std::string getCityDataResource();
    
}
}

#endif
