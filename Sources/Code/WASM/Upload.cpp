// Copyright (c) Asobo Studio, All rights reserved. www.asobostudio.com

#include <MSFS\MSFS.h>
#include <MSFS\MSFS_Render.h>
#include <MSFS\Render\nanovg.h>
#include <MSFS\Legacy\gauges.h>
#include <MSFS\MSFS_Network.h>

#include <map>
#include <math.h>
#include <string>
#include <sys/stat.h>

struct sNetworkUploadInfo
{
	bool bNeedUpdate;
	int m_iFont;
	int m_iJsonUpload;
	std::string m_sJsonPath;
};

static sNetworkUploadInfo g_NetworkUploadInfo;
static std::map<FsContext, NVGcontext*> g_NetworkUploadNVGcontext;

void HttpPut_CopyDataAsJson(FsNetworkRequestId requestId, int errorCode, void* userData)
{
	static unsigned jsonCount = 0;
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

		mkdir("\\work\\Network_Put", -1);
		char path[32] = { 0 };
		sprintf(path, "\\work\\Network_Put\\data%d.json", jsonCount++);
		g_NetworkUploadInfo.m_sJsonPath = path;

		FILE* f = fopen(path, "w");
		if (f)
		{
			fwrite(data, sizeof(char), dataSize, f);
			fclose(f);
		}
	
		free(data);
		g_NetworkUploadInfo.bNeedUpdate = true;
	}
}

void PutRequest()
{
	FsNetworkHttpRequestParam param;

	param.postField = nullptr;
	param.headerOptionsSize = 1;
	param.headerOptions = (char**)calloc(1, sizeof(char*));

	char headerContentType[32] = "accept: application/json";
	param.headerOptions[0] = headerContentType;

	const unsigned dataSize = 64;
	const char* cData = "{\"key1\":\"value1\", \"key2\":\"value2\"}";

	unsigned char data[dataSize] = { 0 };
	strcpy((char*)data, cData);

	param.data = data;
	param.dataSize = strlen(cData);

	fsNetworkHttpRequestPut("https://httpbin.org/anything", &param, HttpPut_CopyDataAsJson, nullptr);

	free(param.headerOptions);
}

extern "C" {

	MSFS_CALLBACK bool Upload_gauge_callback(FsContext ctx, int service_id, void* pData)
	{
		switch (service_id)
		{
		case PANEL_SERVICE_PRE_INSTALL:
		{
			return true;
		}
		break;

		case PANEL_SERVICE_POST_INSTALL:
		{
			NVGparams params;
			params.userPtr = ctx;
			params.edgeAntiAlias = true;
			g_NetworkUploadNVGcontext[ctx] = nvgCreateInternal(&params);
			NVGcontext* nvgctx = g_NetworkUploadNVGcontext[ctx];
			g_NetworkUploadInfo.m_iFont = nvgCreateFont(nvgctx, "sans", "./data/Roboto-Regular.ttf");
			g_NetworkUploadInfo.m_iJsonUpload = 0;
			g_NetworkUploadInfo.bNeedUpdate = true;
			return true;
		}
		break;

		case PANEL_SERVICE_PRE_DRAW:
		{
			if (!g_NetworkUploadInfo.bNeedUpdate)
				return true;

			g_NetworkUploadInfo.bNeedUpdate = false;

			sGaugeDrawData* p_draw_data = (sGaugeDrawData*)pData;
			NVGcontext* nvgctx = g_NetworkUploadNVGcontext[ctx];
			float fSize = sqrt(p_draw_data->winWidth * p_draw_data->winWidth + p_draw_data->winHeight * p_draw_data->winHeight) * 1.1f;
			float pxRatio = (float)p_draw_data->fbWidth / (float)p_draw_data->winWidth;

			nvgBeginFrame(nvgctx, p_draw_data->winWidth, p_draw_data->winHeight, pxRatio);
			nvgFillColor(nvgctx, nvgRGB(0, 0, 0));

			if (g_NetworkUploadInfo.m_sJsonPath.size() != 0)
			{
				nvgRect(nvgctx, 0, 0, p_draw_data->winWidth, p_draw_data->winHeight);
				nvgFillColor(nvgctx, nvgRGBA(0, 0, 0, 255));
				nvgFill(nvgctx);

				nvgTextBounds(nvgctx, 0, 0, nullptr, nullptr, nullptr);
				nvgFontSize(nvgctx, 90.f);
				nvgFontFace(nvgctx, "sans");
				nvgFillColor(nvgctx, nvgRGBA(255, 255, 255, 255));
				nvgTextAlign(nvgctx, NVG_ALIGN_CENTER | NVG_ALIGN_MIDDLE);
				nvgText(nvgctx, p_draw_data->winWidth / 2, p_draw_data->winHeight / 2, g_NetworkUploadInfo.m_sJsonPath.c_str(), nullptr);
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
			NVGcontext* nvgctx = g_NetworkUploadNVGcontext[ctx];

			if (g_NetworkUploadInfo.m_iFont != 0)
			{
				nvgDeleteImage(nvgctx, g_NetworkUploadInfo.m_iFont);
				g_NetworkUploadInfo.m_iFont = 0;
			}

			nvgDeleteInternal(nvgctx);
			g_NetworkUploadNVGcontext.erase(ctx);
			return true;
		}
		break;
		}
		return false;
	}

	MSFS_CALLBACK void Upload_mouse_callback(float fX, float fY, unsigned int iFlags)
	{
		switch (iFlags)
		{
		case MOUSE_LEFTSINGLE:
		case MOUSE_RIGHTSINGLE:
			PutRequest();
			break;
		}
	}
}
