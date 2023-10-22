// Copyright (c) Asobo Studio, All rights reserved. www.asobostudio.com

#include "GetGauge.h"

#include <MSFS\MSFS.h>
#include <MSFS\MSFS_Render.h>
#include <MSFS\Render\nanovg.h>
#include <MSFS\Legacy\gauges.h>
#include <MSFS\MSFS_Network.h>

#include <map>
#include <math.h>
#include <set>
#include <sys/stat.h>

static sNetworkGetInfo g_NetworkGetInfo;
static std::map <FsContext, NVGcontext*> g_NetworkGetNVGcontext;

static void HttpGet_CopyDataAsPng(FsNetworkRequestId requestId, int errorCode, void* userData)
{
	static unsigned pngCount = 0;
	if (errorCode != 200)
	{
		return;
	}
	
	unsigned long dataSize = fsNetworkHttpRequestGetDataSize(requestId);
	if (dataSize > 0 && dataSize != -1)
	{
		unsigned char* data = fsNetworkHttpRequestGetData(requestId);
		if (!data)
		{
			return;
		}
		mkdir("\\work\\Network_GetCallback", -1);
	
		char path[32] = { 0 };
		sprintf(path, "\\work\\Network_GetCallback\\img%d.jpeg", pngCount++);
	
		FILE* f = fopen(path, "w");
		if (f)
		{
			fwrite(data, sizeof(char), dataSize, f);
			fclose(f);
		}
	
		g_NetworkGetInfo.imagePath = path;

		if (g_NetworkGetInfo.imageData)
		{
			free(g_NetworkGetInfo.imageData);
		}
		g_NetworkGetInfo.imageData = data;
		g_NetworkGetInfo.imageDataSize = dataSize;
		g_NetworkGetInfo.bNeedUpdate = true;
	}
}

void DownloadNewImage()
{
	char url[64] = { 0 };
	if (g_NetworkGetInfo.m_uImageWidth == 0 || g_NetworkGetInfo.m_uImageWidth >= 5000 ||
		g_NetworkGetInfo.m_uImageWidth == 0 || g_NetworkGetInfo.m_uImageWidth >= 5000)
	{
		sprintf(url, "https://picsum.photos/500");
	}
	else
	{
		sprintf(url, "https://picsum.photos/%d/%d", g_NetworkGetInfo.m_uImageWidth, g_NetworkGetInfo.m_uImageHeight);
	}

	fsNetworkHttpRequestGet(url, nullptr, HttpGet_CopyDataAsPng, nullptr);
}

extern "C" {

	MSFS_CALLBACK bool GetCallback_gauge_callback(FsContext ctx, int service_id, void* pData)
	{
		switch (service_id)
		{
		case PANEL_SERVICE_PRE_INSTALL:
		{
			sGaugeInstallData* p_install_data = (sGaugeInstallData*)pData;

			g_NetworkGetInfo.m_uImageWidth = p_install_data->iSizeX;
			g_NetworkGetInfo.m_uImageHeight = p_install_data->iSizeY;

			return true;
		}
		break;
		case PANEL_SERVICE_POST_INSTALL:
		{
			NVGparams params;
			params.userPtr = ctx;
			params.edgeAntiAlias = true;
			g_NetworkGetNVGcontext[ctx] = nvgCreateInternal(&params);
			NVGcontext* nvgctx = g_NetworkGetNVGcontext[ctx];
			g_NetworkGetInfo.m_iFont = nvgCreateFont(nvgctx, "sans", "./data/Roboto-Regular.ttf");

			g_NetworkGetInfo.bNeedUpdate = true;

			return true;
		}
		break;

		case PANEL_SERVICE_PRE_DRAW:
		{
			if (!g_NetworkGetInfo.bNeedUpdate)
				return true;

			g_NetworkGetInfo.bNeedUpdate = false;

			sGaugeDrawData* p_draw_data = (sGaugeDrawData*)pData;
			NVGcontext* nvgctx = g_NetworkGetNVGcontext[ctx];
			float fSize = sqrt(p_draw_data->winWidth * p_draw_data->winWidth + p_draw_data->winHeight * p_draw_data->winHeight) * 1.1f;
			float pxRatio = (float)p_draw_data->fbWidth / (float)p_draw_data->winWidth;

			nvgBeginFrame(nvgctx, p_draw_data->winWidth, p_draw_data->winHeight, pxRatio);
			nvgFillColor(nvgctx, nvgRGB(0, 0, 0));

			if (g_NetworkGetInfo.imagePath.size() != 0)
			{
				if (g_NetworkGetInfo.m_iImage != 0)
				{
					nvgDeleteImage(nvgctx, g_NetworkGetInfo.m_iImage);
				}
				g_NetworkGetInfo.m_iImage = nvgCreateImageMem(nvgctx, 0, g_NetworkGetInfo.imageData, g_NetworkGetInfo.imageDataSize);
				//g_NetworkGetInfo.m_iImage = nvgCreateImage(nvgctx, g_NetworkGetInfo.imagePath.c_str(), 0);

				int imgw, imgh;
				nvgImageSize(nvgctx, g_NetworkGetInfo.m_iImage, &imgw, &imgh);
				NVGpaint imgPaint = nvgImagePattern(nvgctx, 0, 0, imgw, imgh, 0, g_NetworkGetInfo.m_iImage, 1);

				nvgBeginPath(nvgctx);
				nvgRoundedRect(nvgctx, 0, 0, imgw, imgh, 5);
				nvgFillPaint(nvgctx, imgPaint);
				nvgFill(nvgctx);
			}
			else
			{
				nvgTextBounds(nvgctx, 0, 0, nullptr, nullptr, nullptr);
				nvgFontSize(nvgctx, 200.0f);
				nvgFontFace(nvgctx, "sans");
				nvgFillColor(nvgctx, nvgRGBA(255, 255, 255, 255));
				nvgTextAlign(nvgctx, NVG_ALIGN_CENTER | NVG_ALIGN_MIDDLE);
				nvgText(nvgctx, p_draw_data->winWidth / 2, p_draw_data->winHeight / 2, "Click Here", nullptr);
			}

			nvgEndFrame(nvgctx);
			return true;
		}
			break;

		case PANEL_SERVICE_PRE_KILL:
		{
			NVGcontext* nvgctx = g_NetworkGetNVGcontext[ctx];

			if (g_NetworkGetInfo.m_iImage != 0)
			{
				nvgDeleteImage(nvgctx, g_NetworkGetInfo.m_iImage);
				g_NetworkGetInfo.m_iImage = 0;
			}

			if (g_NetworkGetInfo.m_iFont != 0)
			{
				nvgDeleteImage(nvgctx, g_NetworkGetInfo.m_iFont);
				g_NetworkGetInfo.m_iFont = 0;
			}

			g_NetworkGetInfo.imagePath.clear();
			nvgDeleteInternal(nvgctx);
			g_NetworkGetNVGcontext.erase(ctx);
			return true;
		}
		break;
		}
		return false;
	}

	MSFS_CALLBACK void GetCallback_mouse_callback(float fX, float fY, unsigned int iFlags)
	{
		switch (iFlags)
		{
		case MOUSE_LEFTSINGLE:
		case MOUSE_RIGHTSINGLE:
			DownloadNewImage();
			break;
		}
	}
}
