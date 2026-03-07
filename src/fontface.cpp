#include "fontface.h"
#include <fstream>

gen::FontFace::FontFace() {
	
}

gen::FontFace::FontFace(std::string datafilename) : _defaultFont("Arial") {
	std::ifstream file(datafilename);
	_jsonData = nlohmann::json::parse(file);

	if (_jsonData.contains(_defaultFont)) {
	    _fontFace = _defaultFont;
	} else {
		for (auto& [key, value] : _jsonData.items()) {
		    _fontFace = key;
		    _defaultFont = key;
		    break;
		}
	}

	for (auto& [key, value] : _jsonData[_fontFace].items()) {
	    _fontSize = key;
	    _defaultFontSize = key;
	    break;
	}
}

std::string gen::FontFace::getFontFace() {
	return _fontFace;
}

bool gen::FontFace::setFontFace(std::string name) {
	if (!_jsonData.contains(name)) {
	    return false;
	}

	_fontFace = name;
	if (!_jsonData[_fontFace].contains(_fontSize)) {
	    for (auto& [key, value] : _jsonData[_fontFace].items()) {
		    _fontSize = key;
		    break;
		}
	}

	return true;
}

bool gen::FontFace::setFontFace(std::string name, int size) {
	if (!_jsonData.contains(name)) {
	    return false;
	}

	std::string sizestr = _intToString(size);
	if (!_jsonData[_fontFace].contains(sizestr)) {
	    return false;
	}

	_fontFace = name;
	_fontSize = sizestr;

	return true;
}

bool gen::FontFace::setFontFace() {
	_fontFace = _defaultFont;
	_fontSize = _defaultFontSize;

	return true;
}

std::vector<std::string> gen::FontFace::getFontFaces() {
	std::vector<std::string> fonts;
	for (auto& [key, value] : _jsonData.items()) {
	    fonts.push_back(key);
	}

	return fonts;
}

std::vector<int> gen::FontFace::getFontSizes() {
	std::vector<int> sizes;
	for (auto& [key, value] : _jsonData[_fontFace].items()) {
	    sizes.push_back(_stringToInt(key));
	}

	return sizes;
}

std::vector<int> gen::FontFace::getFontSizes(std::string font) {
	if (!_jsonData.contains(font)) {
	    return std::vector<int>();
	}

	std::vector<int> sizes;
	for (auto& [key, value] : _jsonData[font].items()) {
	    sizes.push_back(_stringToInt(key));
	}

	return sizes;
}

int gen::FontFace::getFontSize() {
	return _stringToInt(_fontSize);
}

bool gen::FontFace::setFontSize(int size) {
	std::string sizestr = _intToString(size);
	if (!_jsonData[_fontFace].contains(sizestr)) {
	    return false;
	}
	_fontSize = sizestr;

	return true;
}

bool gen::FontFace::setFontSize() {
	for (auto& [key, value] : _jsonData[_fontFace].items()) {
	    _fontSize = key;
	    return true;
	}

	return false;
}

gen::TextExtents gen::FontFace::getTextExtents(std::string str) {
	TextExtents extents;
	if (str.size() == 0) {
		return extents;
	}

	if (str.size() == 1) {
		return _getCharExtents(str[0]);
	}

	TextExtents temp = _getCharExtents(str[0]);
	extents.offx = temp.offx;

	double ymin = 0.0;
	double ymax = 0.0;
	double dx = 0.0;
	for (unsigned int i = 0; i < str.size(); i++) {
		temp = _getCharExtents(str[i]);
	    ymin = fmin(ymin, temp.offy);
	    ymax = fmax(ymax, temp.offy + temp.height);
	    dx += temp.dx;
	}
	extents.offy = ymin;
	
	temp = _getCharExtents(str[str.size() - 1]);
	extents.width = dx + extents.offx - (temp.dx - temp.width);
	extents.height = ymax - ymin;
	extents.dx = dx;
	extents.dy = 0.0;

	return extents;
}

std::vector<gen::TextExtents> gen::FontFace::getCharacterExtents(std::string str) {
	std::vector<TextExtents> extents;
	extents.reserve(str.size());
	TextExtents charExtents;
	double x = 0;
	for (unsigned int i = 0; i < str.size(); i++) {
		charExtents = _getCharExtents(str[i]);

		charExtents.offx += x;
		x += charExtents.dx;

		extents.push_back(charExtents);
	}

	return extents;
}

gen::TextExtents gen::FontFace::_getCharExtents(char c) {
	std::string key(1, c);
	auto& chardata = _jsonData[_fontFace][_fontSize][key];
	
	TextExtents extents;
	extents.offx = chardata[0].get<double>();
	extents.offy = chardata[1].get<double>();
	extents.width = chardata[2].get<double>();
	extents.height = chardata[3].get<double>();
	extents.dx = chardata[4].get<double>();
	extents.dy = chardata[5].get<double>();

	return extents;
}

std::string gen::FontFace::_intToString(int number) {
    std::ostringstream ss;
    ss << number;
    return ss.str();
}

int gen::FontFace::_stringToInt(std::string number) {
	int s = 0;
	std::istringstream(number) >> s;
	return s;
}
