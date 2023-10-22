#pragma once

#include <string>

struct sNetworkGetInfo
{
	bool bNeedUpdate;
	std::string imagePath;
	unsigned char* imageData;
	unsigned imageDataSize;

	int m_uImageWidth;
	int m_uImageHeight;

	int m_iFont;
	int m_iImage;
};
