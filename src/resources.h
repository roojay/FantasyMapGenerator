#ifndef RESOURCES_H
#define RESOURCES_H

#define RESOURCES_EXECUTABLE_DIRECTORY 	"/tmp/cpp_build"
#define RESOURCES_FONT_DATA_DIRECTORY 	"/tmp/cpp_build/fontdata"
#define RESOURCES_CITY_DATA_DIRECTORY 	"/tmp/cpp_build/citydata"
#define RESOURCES_FONT_DATA_RESOURCE 	"/tmp/cpp_build/fontdata/fontdata.json"
#define RESOURCES_CITY_DATA_RESOURCE 	"/tmp/cpp_build/citydata/countrycities.json"

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
